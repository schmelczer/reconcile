mod edited_text;
mod merge_context;
mod operation;

pub use edited_text::EditedText;
pub use operation::Operation;

use crate::tokenizer::Tokenizer;

pub fn reconcile(original: &str, left: &str, right: &str) -> String {
    let left_operations = EditedText::from_strings(original, left);
    let right_operations = EditedText::from_strings(original, right);

    let merged_operations = left_operations.merge(right_operations);
    merged_operations.apply()
}

pub fn reconcile_with_tokenizer<F, T>(
    original: &str,
    left: &str,
    right: &str,
    tokenizer: &Tokenizer<T>,
) -> String
where
    T: PartialEq + Clone,
{
    let left_operations = EditedText::from_strings_with_tokenizer(original, left, tokenizer);
    let right_operations = EditedText::from_strings_with_tokenizer(original, right, tokenizer);

    let merged_operations = left_operations.merge(right_operations);
    merged_operations.apply()
}

#[cfg(test)]
mod test {
    use std::{fs, ops::Range, path::Path};

    use pretty_assertions::assert_eq;

    use test_case::test_matrix;

    use super::*;

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

        // One deleted a large range, the other deleted subranges and inserted as well
        test_merge_both_ways(
            "original_1 original_2 original_3 original_4 original_5",
            "original_1 original_5",
            "original_1 edit_1 original_3 edit_2 original_5",
            "original_1 edit_1 edit_2 original_5",
        );

        // One deleted a large range, the other inserted and deleted a partially overlapping range
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

        // test_merge_both_ways("hello world", "world !", "hi hello world", "hi world !");

        test_merge_both_ways(
            "both delete the same word",
            "both the same word",
            "both the same word",
            "both the same word",
        );

        test_merge_both_ways("    ", "it’s utf-8!", "   ", "it’s utf-8!");

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

    #[test_matrix( [
        "pride_and_prejudice.txt",
        "romeo_and_juliet.txt",
        "room_with_a_view.txt",
        "kun_lu.txt",

    ],  [
        "pride_and_prejudice.txt",
        "romeo_and_juliet.txt",
        "room_with_a_view.txt",
        "kun_lu.txt"
    ],  [
        "pride_and_prejudice.txt",
        "romeo_and_juliet.txt",
        "room_with_a_view.txt",
        "kun_lu.txt"
    ], [0..10000, 10000..20000], [0..10000, 10000..20000], [0..10000, 10000..20000])]
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

        let root = Path::new("test/resources/");

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

        reconcile(&contents[0], &contents[1], &contents[2]);
    }

    fn test_merge_both_ways(original: &str, edit_1: &str, edit_2: &str, expected: &str) {
        assert_eq!(reconcile(original, edit_1, edit_2), expected);
        assert_eq!(reconcile(original, edit_2, edit_1), expected);
    }
}
