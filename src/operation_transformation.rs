mod edited_text;
mod operation;
mod utils;
mod transport;
use std::fmt::Debug;


pub use transport::{ChangeSet};
pub use operation::Operation;
pub use edited_text::{EditedText};

use crate::{Tokenizer, types::text_with_cursors::TextWithCursors};

/// Given an `original` document and two concurrent edits to it,
/// return a document containing all changes from both `left`
/// and `right`.
///
/// If a span has been inserted in either the `left` or `right`
/// versions, it will be present in the return value. If both sides
/// insert the same span with a common prefix, that prefix will only
/// be present once in the output.
///
/// When both sides delete the same span, it will be deleted in the
/// return value. If one side deletes a span and the other side inserts
/// into that span, the inserted text will be present in the return
/// value.
///
/// The function supports UTF-8. The arguments are tokenized at the
/// granularity of words.
///
/// ```
/// use reconcile_text::{reconcile, BuiltinTokenizer};
///
/// let parent = "Merging text is hard!";
/// let left = "Merging text is easy!";
/// let right = "With reconcile, merging documents is hard!";
///
/// let deconflicted = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Word);
/// assert_eq!(deconflicted.apply().text(), "With reconcile, merging documents is easy!");
/// ```
#[must_use]
pub fn reconcile<'a, T>(
    original: &'a str,
    left: &TextWithCursors,
    right: &TextWithCursors,
    tokenizer: &Tokenizer<T>,
) -> EditedText<'a, T>
where
    T: PartialEq + Clone + Debug,
{
    let left_operations = EditedText::from_strings_with_tokenizer(original, left, tokenizer);
    let right_operations = EditedText::from_strings_with_tokenizer(original, right, tokenizer);

    left_operations.merge(right_operations)
}

#[cfg(test)]
mod test {
    use std::{fs, ops::Range, path::Path};

    use pretty_assertions::assert_eq;
    use test_case::test_matrix;

    use super::*;
    use crate::{BuiltinTokenizer, CursorPosition, types::text_with_cursors::TextWithCursors};

    #[test]
    fn test_cursor_complex() {
        let original: &'static str = "this is some complex text to test cursor positions";
        let left = TextWithCursors::new(
            "this is really complex text for testing cursor positions".to_owned(),
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
            "that was some complex sample to test cursor movements".to_owned(),
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

        let merged = reconcile(original, &left, &right, &*BuiltinTokenizer::Word).apply();
        assert_eq!(
            &merged.text(),
            "that was really complex sample for testing cursor movements"
        );
        assert_eq!(
            merged.cursors(),
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
                    char_index: 30
                }, // after "complex sample"
            ]
        );
    }

    #[ignore = "expensive to run, only run in CI"]
    #[test_matrix( [
        "pride_and_prejudice.txt",
        "room_with_a_view.txt",
        "kun_lu.txt",
        "blns.txt"
    ],  [
        "pride_and_prejudice.txt",
        "room_with_a_view.txt",
    ],  [
        "room_with_a_view.txt",
        "kun_lu.txt",
        "blns.txt"
    ], [0..10000], [0..10000, 10000..20000], [0..10000, 10000..20000])]
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

        let _ = reconcile(
            &contents[0],
            &(&contents[1]).into(),
            &(&contents[2]).into(),
            &*BuiltinTokenizer::Word,
        );
    }
}
