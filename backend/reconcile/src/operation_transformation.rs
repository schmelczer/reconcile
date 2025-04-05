mod cursor;
mod edited_text;
mod merge_context;
mod operation;
mod ordered_operation;

pub use cursor::{CursorPosition, TextWithCursors};
pub use edited_text::EditedText;
pub use operation::Operation;

use crate::Tokenizer;

#[must_use]
pub fn reconcile(original: &str, left: &str, right: &str) -> String {
    reconcile_with_cursors(original, left.into(), right.into())
        .text
        .to_string()
}

#[must_use]
pub fn reconcile_with_cursors<'a>(
    original: &'a str,
    left: TextWithCursors<'a>,
    right: TextWithCursors<'a>,
) -> TextWithCursors<'static> {
    let left_operations = EditedText::from_strings(original, left);
    let right_operations = EditedText::from_strings(original, right);

    let merged_operations = left_operations.merge(right_operations);

    TextWithCursors::new_owned(merged_operations.apply(), merged_operations.cursors)
}

#[must_use]
pub fn reconcile_with_tokenizer<'a, F, T>(
    original: &str,
    left: TextWithCursors<'a>,
    right: TextWithCursors<'a>,
    tokenizer: &Tokenizer<T>,
) -> TextWithCursors<'static>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    let left_operations = EditedText::from_strings_with_tokenizer(original, left, tokenizer);
    let right_operations = EditedText::from_strings_with_tokenizer(original, right, tokenizer);

    let merged_operations = left_operations.merge(right_operations);

    TextWithCursors::new_owned(merged_operations.apply(), merged_operations.cursors)
}

#[cfg(test)]
mod test {
    use std::{fs, ops::Range, path::Path};

    use pretty_assertions::assert_eq;
    use test_case::test_matrix;

    use super::*;
    use crate::CursorPosition;

    #[test]
    fn test_cursor_complex() {
        let original = "this is some complex text to test cursor positions";
        let left = TextWithCursors::new(
            "this is really complex text for testing cursor positions",
            vec![
                CursorPosition {
                    id: 0,
                    char_index: 8,
                }, // after "this is "
                CursorPosition {
                    id: 1,
                    char_index: 22,
                }, // after "this is really complex text"
            ],
        );
        let right = TextWithCursors::new(
            "that was some complex sample to test cursor movements",
            vec![
                CursorPosition {
                    id: 2,
                    char_index: 5,
                }, // after "that "
                CursorPosition {
                    id: 3,
                    char_index: 29,
                }, // after "some complex sample "
            ],
        );

        let merged = reconcile_with_cursors(original, left, right);

        assert_eq!(
            merged,
            TextWithCursors::new(
                "that was really complex sample for testing cursor movements",
                vec![
                    CursorPosition {
                        id: 2,
                        char_index: 5
                    }, // unchanged
                    CursorPosition {
                        id: 0,
                        char_index: 9
                    }, // before "really"
                    CursorPosition {
                        id: 1,
                        char_index: 23
                    }, // inside of "s|ample" because "text" got replaced by "sample"
                    CursorPosition {
                        id: 3,
                        char_index: 31
                    }, // before "for"
                ]
            )
        );
    }

    #[ignore = "expensive to run, only run in CI"]
    #[test_matrix( [
        "pride_and_prejudice.txt",
        "romeo_and_juliet.txt",
        "room_with_a_view.txt",
        "kun_lu.txt",
        "blns.txt"
    ],  [
        "pride_and_prejudice.txt",
        "romeo_and_juliet.txt",
        "room_with_a_view.txt",
        "kun_lu.txt",
        "blns.txt"
    ],  [
        "pride_and_prejudice.txt",
        "romeo_and_juliet.txt",
        "room_with_a_view.txt",
        "kun_lu.txt",
        "blns.txt"
    ], [0..10000, 10000..20000, 20000..50000], [0..10000, 10000..20000, 20000..50000], [0..10000, 10000..20000, 20000..50000])]
    fn test_merge_files_without_panic(
        file_name_1: &str,
        file_name_2: &str,
        file_name_3: &str,
        range_1: Range<usize>,
        range_2: Range<usize>,
        range_3: Range<usize>,
    ) {
        let files = [file_name_1, file_name_2, file_name_3];
        let permutations = [range_1, range_2, range_3];

        let root = Path::new("tests/resources/");

        let contents = files
            .iter()
            .zip(permutations.iter())
            .map(|(file, range)| {
                let path = root.join(file);
                fs::read_to_string(&path)
                    .unwrap()
                    .chars()
                    .skip(range.start)
                    .take(range.end)
                    .collect::<String>()
            })
            .collect::<Vec<_>>();

        let _ = reconcile(&contents[0], &contents[1], &contents[2]);
    }
}
