use std::cmp::Ordering;

use super::Operation;
use crate::diffs::myers::diff;
use crate::diffs::raw_operation::RawOperation;
use crate::errors::SyncLibError;
use crate::tokenizer::token::Token;
use ropey::Rope;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
struct MergeContext {
    last_delete: Option<Operation>,
    shift: i64,
}

/// A sequence of operations that can be applied to a text document.
/// OperationSequence supports merging two sequences of operations using the
/// principle of Operational Transformation.
///
/// It's mainly created through the from_strings method, then merged with another
/// OperationSequence derived from the same original text and then applied to the original text
/// to get the reconciled text of concurrent edits.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct OperationSequence {
    operations: Vec<Operation>,
}

impl OperationSequence {
    /// Creates a new OperationSequence with the given operations.
    /// The operations should be in the order they should be applied.
    /// The operations must not overlap.
    pub fn new(operations: Vec<Operation>) -> Self {
        operations
            .iter()
            .zip(operations.iter().skip(1))
            .for_each(|(previous, next)| {
                debug_assert!(
                    previous.start_index() <= next.start_index(),
                    "{} doesn't come before {}",
                    previous,
                    next
                );
            });

        Self { operations }
    }

    /// Creates an OperationSequence from the given original (old) and updated (new) strings.
    /// The returned OperationSequence represents the changes from the original to the updated text.
    /// When the return value is applied to the original text, it will result in the updated text.
    pub fn from_strings(original: &str, updated: &str) -> Self {
        let original_tokens = Token::tokenize(original);
        let updated_tokens = Token::tokenize(updated);

        let diff: Vec<RawOperation> = diff(&original_tokens, &updated_tokens);

        Self::new(Self::raw_operations_to_operations(diff))
    }

    fn raw_operations_to_operations(raw_operations: Vec<RawOperation>) -> Vec<Operation> {
        let mut index = 0;
        raw_operations
            .into_iter()
            .flat_map(|raw_operation| {
                match raw_operation {
                    RawOperation::Equal(..) => {
                        index += raw_operation.original_text_length();
                        None
                    }
                    RawOperation::Insert(..) => {
                        let length = raw_operation.original_text_length();
                        let result =
                            Operation::create_insert(index, raw_operation.get_original_text());
                        index += length;
                        result
                    }
                    RawOperation::Delete(..) => {
                        Operation::create_delete_with_text(index, raw_operation.get_original_text())
                    }
                }
                .into_iter()
            })
            .collect()
    }

    pub fn merge(&self, other: &Self) -> Result<Self, SyncLibError> {
        let mut merged_operations =
            Vec::with_capacity(self.operations.len() + other.operations.len());

        let mut left_merge_context = MergeContext::default();
        let mut right_merge_context = MergeContext::default();

        let mut left_index: usize = 0;
        let mut right_index: usize = 0;

        loop {
            let shifted_left_op = self
                .operations
                .get(left_index)
                .map(|op| {
                    Self::pick_up_dangling_delete_from_affecting_context(
                        op,
                        &mut right_merge_context,
                    );
                    op.with_shifted_index(right_merge_context.shift)
                })
                .transpose()?;

            let shifted_right_op = other
                .operations
                .get(right_index)
                .map(|op| {
                    Self::pick_up_dangling_delete_from_affecting_context(
                        op,
                        &mut left_merge_context,
                    );
                    op.with_shifted_index(left_merge_context.shift)
                })
                .transpose()?;

            let left_op_index = shifted_left_op
                .as_ref()
                .map(|op| {
                    op.start_index().max(
                        left_merge_context
                            .last_delete
                            .as_ref()
                            .map(|op| op.end_index())
                            .unwrap_or_default(),
                    ) as i64
                })
                .unwrap_or_default();

            let right_op_index = shifted_right_op
                .as_ref()
                .map(|op| {
                    op.start_index().max(
                        right_merge_context
                            .last_delete
                            .as_ref()
                            .map(|op| op.end_index())
                            .unwrap_or_default(),
                    ) as i64
                })
                .unwrap_or_default();

            let result = left_op_index.cmp(&right_op_index);
            let order = if result == Ordering::Equal
                && shifted_left_op.is_some()
                && shifted_right_op.is_some()
            {
                match (
                    shifted_left_op.as_ref().unwrap(),
                    shifted_right_op.as_ref().unwrap(),
                ) {
                    (Operation::Insert { .. }, Operation::Delete { .. }) => Ordering::Greater,
                    (Operation::Delete { .. }, Operation::Insert { .. }) => Ordering::Less,
                    _ => Ordering::Equal,
                }
            } else {
                result
            };

            match (shifted_left_op, shifted_right_op, order) {
                (Some(left_op), None, _)
                | (Some(left_op), Some(_), std::cmp::Ordering::Less | std::cmp::Ordering::Equal) => {
                    if let Some(op) = Self::merge_operations_with_context(
                        left_op,
                        &mut right_merge_context,
                        &mut left_merge_context,
                    )? {
                        merged_operations.push(op);
                    }

                    left_index += 1;
                }
                (None, Some(right_op), _)
                | (Some(_), Some(right_op), std::cmp::Ordering::Greater) => {
                    if let Some(op) = Self::merge_operations_with_context(
                        right_op,
                        &mut left_merge_context,
                        &mut right_merge_context,
                    )? {
                        merged_operations.push(op);
                    }

                    right_index += 1;
                }
                (None, None, _) => {
                    break;
                }
            };
        }

        Ok(Self::new(merged_operations))
    }

    pub fn apply<'a>(&self, rope_text: &'a mut Rope) -> Result<&'a mut Rope, SyncLibError> {
        for operation in &self.operations {
            operation.apply(rope_text)?;
        }

        Ok(rope_text)
    }

    fn merge_operations_with_context(
        aligned_operation: Operation,
        affecting_context: &mut MergeContext,
        produced_context: &mut MergeContext,
    ) -> Result<Option<Operation>, SyncLibError> {
        Ok(
            match (aligned_operation, affecting_context.last_delete.clone()) {
                (operation @ Operation::Insert { .. }, None) => {
                    produced_context.shift += operation.len() as i64;
                    Some(operation)
                }

                (operation @ Operation::Delete { .. }, None) => {
                    Self::replace_delete_in_produced_context(
                        produced_context,
                        Some(operation.clone()),
                    );
                    Some(operation)
                }

                (operation @ Operation::Insert { .. }, Some(last_delete)) => {
                    produced_context.shift += operation.len() as i64;

                    if last_delete.range().contains(&operation.start_index()) {
                        let moved_operation = operation.with_index(last_delete.start_index());

                        affecting_context.last_delete = Operation::create_delete(
                            moved_operation.end_index() + 1,
                            last_delete.len(),
                        );

                        Some(moved_operation)
                    } else {
                        Some(operation)
                    }
                }

                (operation @ Operation::Delete { .. }, Some(last_delete)) => {
                    let updated_delete = if last_delete.range().contains(&operation.start_index()) {
                        let overlap =
                            last_delete.end_index() as i64 - operation.start_index() as i64 + 1;

                        affecting_context.last_delete = Operation::create_delete(
                            last_delete.start_index(),
                            0.max(last_delete.len() as i64 - operation.len() as i64) as usize,
                        );

                        if last_delete.end_index() < operation.end_index() {
                            affecting_context.shift -= last_delete.len() as i64 - overlap
                        }

                        Operation::create_delete(
                            last_delete.start_index(),
                            0.max(operation.len() as i64 - overlap) as usize,
                        )
                    } else {
                        Some(operation)
                    };

                    Self::replace_delete_in_produced_context(
                        produced_context,
                        updated_delete.clone(),
                    );
                    updated_delete
                }
            },
        )
    }

    fn replace_delete_in_produced_context(
        produced_context: &mut MergeContext,
        delete: Option<Operation>,
    ) {
        if let Some(produced_last_delete) = produced_context.last_delete.take() {
            produced_context.shift -= produced_last_delete.len() as i64;
        }

        produced_context.last_delete = delete;
    }

    fn pick_up_dangling_delete_from_affecting_context(
        next_operation: &Operation,
        affecting_context: &mut MergeContext,
    ) {
        match affecting_context.last_delete.as_ref() {
            Some(last_delete)
                if next_operation.start_index() as i64 + affecting_context.shift
                    > last_delete.end_index() as i64 =>
            {
                affecting_context.shift -= last_delete.len() as i64;
                affecting_context.last_delete = None;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use itertools::Itertools;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_calculate_operations() {
        let left = "hello world! How are you?  Adam";
        let right = "Hello, my friend! How are you doing? Albert";

        let operations = OperationSequence::from_strings(left, right);

        insta::assert_debug_snapshot!(operations);

        let mut left = Rope::from_str(left);
        let new_right = operations.apply(&mut left).unwrap();

        assert_eq!(new_right.to_string(), right);
    }

    #[test]
    fn test_calculate_operations_with_large_diff() {
        let left = "hello world! How are you?  Adam";
        let right = "Hello, my friend! How are you doing? Albert";

        let result = OperationSequence::from_strings(left, right);

        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_calculate_operations_with_no_diff() {
        let left = "hello world!";
        let right = "hello world!";

        let operations = OperationSequence::from_strings(left, right);

        assert_eq!(operations.operations.len(), 0);

        let mut left = Rope::from_str(left);
        let new_right = operations.apply(&mut left).unwrap();

        assert_eq!(new_right.to_string(), right);
    }

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
            "original_1 edit_1 edit_1 original_3",
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
            "both same wordword",
        );

        test_merge_both_ways(
            "long text with one big delete and many small",
            "long small",
            "long with big and small",
            "long small",
        );
    }

    #[test]

    fn test_merge_files_without_panicing() {
        let files = vec![
            "pride_and_prejudice.txt",
            "romeo_and_juliet.txt",
            "room_with_a_view.txt",
        ];

        let root = Path::new("test/resources/");
        let contents = files
            .into_iter()
            .map(|name| fs::read_to_string(root.join(name)).unwrap())
            .map(|text| text[..15000].to_string())
            .collect::<Vec<_>>();

        contents
            .iter()
            .permutations(3)
            .unique()
            .for_each(|permutations| {
                test_merge(permutations[0], permutations[1], permutations[2]);
            });
    }

    fn test_merge_both_ways(original: &str, edit_1: &str, edit_2: &str, expected: &str) {
        assert_eq!(test_merge(original, edit_1, edit_2), expected);
        assert_eq!(test_merge(original, edit_2, edit_1), expected);
    }

    fn test_merge(original: &str, edit_1: &str, edit_2: &str) -> String {
        println!(
            "original: '{:#}'",
            original[..100.min(original.len())].to_string()
        );
        println!(
            "edit_1: '{:#}'",
            edit_1[..100.min(edit_1.len())].to_string()
        );
        println!(
            "edit_2: '{:#}'",
            edit_2[..100.min(edit_2.len())].to_string()
        );

        let mut original = Rope::from_str(original);

        let operations_1 = OperationSequence::from_strings(&original.to_string(), edit_1);
        println!(
            "operations_1: {:?}",
            operations_1.operations[..20.min(operations_1.operations.len())].to_vec()
        );
        let operations_2 = OperationSequence::from_strings(&original.to_string(), edit_2);
        println!(
            "operations_2: {:?}",
            operations_2.operations[..20.min(operations_2.operations.len())].to_vec()
        );

        assert_eq!(
            operations_1
                .apply(&mut original.clone())
                .unwrap()
                .to_string(),
            edit_1
        );
        assert_eq!(
            operations_2
                .apply(&mut original.clone())
                .unwrap()
                .to_string(),
            edit_2
        );

        let merged = operations_1.merge(&operations_2).unwrap();

        let result = merged.apply(&mut original).unwrap();
        result.to_string()
    }
}
