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
    fn test_merges() {
        // Both replaced one token but different
        test_merge_both_ways(
            "original_1 original_2 original_3",
            "original_1 edit_1 original_3",
            "original_1 original_2 edit_2",
            "original_1 edit_1 edit_2",
        );

        // Both replaced the same one token
        test_merge_both_ways(
            "original_1 original_2 original_3",
            "original_1 edit_1 original_3",
            "original_1 edit_1 original_3",
            "original_1 edit_1 original_3",
        );

        // One deleted a large range, the other deleted subranges and inserted as
        // well
        test_merge_both_ways(
            "original_1 original_2 original_3 original_4 original_5",
            "original_1 original_5",
            "original_1 edit_1 original_3 edit_2 original_5",
            "original_1 edit_1 edit_2 original_5",
        );

        // One deleted a large range, the other inserted and deleted a partially
        // overlapping range
        test_merge_both_ways(
            "original_1 original_2 original_3 original_4 original_5",
            "original_1 original_5",
            "original_1 edit_1 original_3 edit_2",
            "original_1 edit_1 edit_2",
        );

        // Merge a replace and an append
        test_merge_both_ways("a b ", "c d ", "a b c d ", "c d c d ");

        test_merge_both_ways("a b c d e", "a e", "a c e", "a e");

        test_merge_both_ways("a 0 1 2 b", "a b", "a E 1 F b", "a E F b");

        test_merge_both_ways(
            "a this one delete b",
            "a b",
            "a my one change b",
            "a my change b",
        );

        test_merge_both_ways(
            "this stays, this is one big delete, don't touch this",
            "this stays, don't touch this",
            "this stays, my one change, don't touch this",
            "this stays, my change, don't touch this",
        );

        test_merge_both_ways("1 2 3 4 5 6", "1 6", "1 2 4 ", "1 ");

        test_merge_both_ways(
            "hello world",
            "hi, world",
            "hello my friend!",
            "hi, my friend!",
        );

        test_merge_both_ways(
            "both delete the same word",
            "both the same word",
            "both the same word",
            "both the same word",
        );

        test_merge_both_ways("    ", "it’s utf-8!", "    ", "it’s utf-8!");

        test_merge_both_ways(
            "both delete the same word but one a bit more",
            "both the same word",
            "both same word",
            "both same word",
        );

        test_merge_both_ways(
            "long text with one big delete and many small",
            "long small",
            "long with big and small",
            "long small",
        );
    }

    #[test]
    fn test_reconcile_idempotent_inserts() {
        // Both inserted the same prefix; this should get deduped
        test_merge_both_ways(
            "hi ",
            "hi there ",
            "hi there my friend ",
            "hi there my friend ",
        );

        // The prefix of the 2nd appears on the 1st so it shouldn't get duplicated
        test_merge_both_ways(
            "hi ",
            "hi there you ",
            "hi there my friend ",
            "hi there my friend you ",
        );

        test_merge_both_ways("a", "a b c", "a b c d", "a b c d");

        test_merge_both_ways(
            "      |7ca2b36d-6ee7-49eb-8eb1-d77e4cc1a001|   ",
              "      |7ca2b36d-6ee7-49eb-8eb1-d77e4cc1a001|      |cd9195cc-103a-4f13-90c8-4fba0ba421ee|      |d39156cc-cfd6-42a8-b70a-75020896069d|      |fbad794c-9c47-41f2-a343-490284ecb5a0|      |dup|   ",
             "       |7ca2b36d-6ee7-49eb-8eb1-d77e4cc1a001|      |cd9195cc-103a-4f13-90c8-4fba0ba421ee|      |dup|   ",
            "       |7ca2b36d-6ee7-49eb-8eb1-d77e4cc1a001|      |cd9195cc-103a-4f13-90c8-4fba0ba421ee|      |d39156cc-cfd6-42a8-b70a-75020896069d|      |fbad794c-9c47-41f2-a343-490284ecb5a0|      |dup|      |dup|   ");
    }

    #[test]
    fn test_cursor_position_no_updates() {
        let original = "hello world";
        let left = TextWithCursors::new(
            "hello world",
            vec![CursorPosition {
                id: 0,
                char_index: 0,
            }],
        );
        let right = TextWithCursors::new(
            "hello world",
            vec![CursorPosition {
                id: 1,
                char_index: 5,
            }],
        );

        let merged = reconcile_with_cursors(original, left, right);

        assert_eq!(
            merged,
            TextWithCursors::new(
                "hello world",
                vec![
                    CursorPosition {
                        id: 0,
                        char_index: 0
                    },
                    CursorPosition {
                        id: 1,
                        char_index: 5
                    }
                ]
            )
        );
    }

    #[test]
    fn test_cursor_position_updates_with_inserts() {
        let original = "hi";
        let left = TextWithCursors::new(
            "hi there",
            vec![CursorPosition {
                id: 0,
                char_index: 7,
            }],
        );
        let right = TextWithCursors::new(
            "hi world!",
            vec![
                CursorPosition {
                    id: 1,
                    char_index: 9,
                },
                CursorPosition {
                    id: 2,
                    char_index: 1,
                },
            ],
        );

        let merged = reconcile_with_cursors(original, left, right);

        assert_eq!(
            merged,
            TextWithCursors::new(
                "hi there world!",
                vec![
                    CursorPosition {
                        id: 2,
                        char_index: 1,
                    },
                    CursorPosition {
                        id: 0,
                        char_index: 7
                    },
                    CursorPosition {
                        id: 1,
                        char_index: 15
                    },
                ]
            )
        );
    }

    #[test]
    fn test_cursor_position_updates_with_deleted() {
        let original = "a b c d";
        let left = TextWithCursors::new(
            "a b d",
            vec![CursorPosition {
                id: 0,
                char_index: 1, // after a
            }],
        );
        let right = TextWithCursors::new(
            "c d",
            vec![CursorPosition {
                id: 1,
                char_index: 1, // after c
            }],
        );

        let merged = reconcile_with_cursors(original, left, right);

        assert_eq!(
            merged,
            TextWithCursors::new(
                " d",
                vec![
                    CursorPosition {
                        id: 0,
                        char_index: 0
                    },
                    CursorPosition {
                        id: 1,
                        char_index: 1
                    }
                ]
            )
        );
    }

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

    fn test_merge_both_ways(original: &str, edit_1: &str, edit_2: &str, expected: &str) {
        assert_eq!(reconcile(original, edit_1, edit_2), expected);
        assert_eq!(reconcile(original, edit_2, edit_1), expected);
    }
}
