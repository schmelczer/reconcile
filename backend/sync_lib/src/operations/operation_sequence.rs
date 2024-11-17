use std::cmp::Ordering;

use super::Operation;
use crate::errors::SyncLibError;
use ropey::Rope;
use serde::{Deserialize, Serialize};
use similar::Algorithm;
use similar::{utils::TextDiffRemapper, ChangeTag, TextDiff};

#[derive(Debug, Clone, Default)]
struct MergeContext {
    previous_delete: Option<Operation>,
    shift: i64,
}

pub fn tokenize(text: &str) -> Vec<&str> {
    text.split_inclusive(|c: char| c.is_whitespace()).collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct OperationSequence {
    operations: Vec<Operation>,
}

impl OperationSequence {
    pub fn new(operations: Vec<Operation>) -> Self {
        Self { operations }
    }

    pub fn try_from_string_diff(
        left: &str,
        right: &str,
        diff_ratio_threshold: f32,
    ) -> Result<Self, SyncLibError> {
        let left_tokens = tokenize(left);
        let right_tokens = tokenize(right);

        let diff = TextDiff::configure()
            .algorithm(Algorithm::Patience)
            .diff_slices(&left_tokens, &right_tokens);

        let diff_ratio = 1.0 - diff.ratio();
        if diff_ratio > diff_ratio_threshold {
            return Err(SyncLibError::DiffTooLarge {
                diff_ratio,
                diff_ratio_limit: diff_ratio_threshold,
            });
        }

        let remapper = TextDiffRemapper::from_text_diff(&diff, left, right);

        let mut index = 0;
        diff.ops()
            .iter()
            .flat_map(move |x| remapper.iter_slices(x))
            .map(|(tag, text)| match tag {
                ChangeTag::Equal => {
                    index += text.chars().count();
                    Ok(None)
                }
                ChangeTag::Insert => {
                    let result = Operation::create_insert(index, text);
                    index += text.chars().count();
                    result
                }
                ChangeTag::Delete => Operation::create_delete(index, text.chars().count()),
            })
            .flat_map(|result| result.transpose().into_iter())
            .collect::<Result<Vec<_>, SyncLibError>>()
            .map(Self::new)
    }

    pub fn apply<'a>(&self, rope_text: &'a mut Rope) -> Result<&'a mut Rope, SyncLibError> {
        for operation in &self.operations {
            operation.apply(rope_text)?;
        }

        Ok(rope_text)
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

            println!();

            let left_op_index = shifted_left_op
                .as_ref()
                .map(|op| {
                    op.start_index().max(
                        left_merge_context
                            .previous_delete
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
                            .previous_delete
                            .as_ref()
                            .map(|op| op.end_index())
                            .unwrap_or_default(),
                    ) as i64
                })
                .unwrap_or_default();

            println!(
                "{:#?} (idx {}) <> {:#?} (idx {})",
                shifted_left_op.clone(),
                left_op_index,
                shifted_right_op.clone(),
                right_op_index
            );

            println!("{:?} <> {:?}", left_merge_context, right_merge_context);

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
                    println!("Left op: {:?}", left_op);

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
                    println!("Right op: {:?}", right_op);

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

            println!("last {:?}", merged_operations.last().unwrap());
            println!("{:?} <> {:?}", left_merge_context, right_merge_context);
        }

        Ok(Self::new(merged_operations))
    }

    fn merge_operations_with_context(
        aligned_operation: Operation,
        affecting_context: &mut MergeContext,
        produced_context: &mut MergeContext,
    ) -> Result<Option<Operation>, SyncLibError> {
        Ok(
            match (aligned_operation, affecting_context.previous_delete.clone()) {
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

                (operation @ Operation::Insert { .. }, Some(previous_delete)) => {
                    produced_context.shift += operation.len() as i64;

                    if previous_delete.range().contains(&operation.start_index()) {
                        let moved_operation =
                            operation.with_index(previous_delete.start_index())?;

                        affecting_context.previous_delete = Operation::create_delete(
                            moved_operation.end_index() + 1,
                            previous_delete.len(),
                        )?;

                        Some(moved_operation)
                    } else {
                        Some(operation)
                    }
                }

                (operation @ Operation::Delete { .. }, Some(previous_delete)) => {
                    let updated_delete = if previous_delete
                        .range()
                        .contains(&operation.start_index())
                    {
                        let overlap =
                            previous_delete.end_index() as i64 - operation.start_index() as i64 + 1;

                        affecting_context.previous_delete = Operation::create_delete(
                            previous_delete.start_index(),
                            0.max(previous_delete.len() as i64 - operation.len() as i64) as usize,
                        )?;

                        if previous_delete.end_index() < operation.end_index() {
                            affecting_context.shift -= previous_delete.len() as i64 - overlap
                        }

                        Operation::create_delete(
                            previous_delete.start_index(),
                            0.max(operation.len() as i64 - overlap) as usize,
                        )?
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
        if let Some(produced_previous_delete) = produced_context.previous_delete.take() {
            produced_context.shift -= produced_previous_delete.len() as i64;
        }

        produced_context.previous_delete = delete;
    }

    fn pick_up_dangling_delete_from_affecting_context(
        next_operation: &Operation,
        affecting_context: &mut MergeContext,
    ) {
        match affecting_context.previous_delete.as_ref() {
            Some(previous_delete)
                if next_operation.start_index() as i64 + affecting_context.shift
                    > previous_delete.end_index() as i64 =>
            {
                affecting_context.shift -= previous_delete.len() as i64;
                affecting_context.previous_delete = None;
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
    fn test_calculate_operations() -> Result<(), SyncLibError> {
        let left = "hello world! How are you?  Adam";
        let right = "Hello, my friend! How are you doing? Albert";

        let operations = OperationSequence::try_from_string_diff(left, right, 0.8)?;

        insta::assert_debug_snapshot!(operations);

        let mut left = Rope::from_str(left);
        let new_right = operations.apply(&mut left)?;

        assert_eq!(new_right.to_string(), right);

        Ok(())
    }

    #[test]
    fn test_calculate_operations_with_large_diff() {
        let left = "hello world! How are you?  Adam";
        let right = "Hello, my friend! How are you doing? Albert";

        let result = OperationSequence::try_from_string_diff(left, right, 0.1);

        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_calculate_operations_with_no_diff() -> Result<(), SyncLibError> {
        let left = "hello world!";
        let right = "hello world!";

        let operations = OperationSequence::try_from_string_diff(left, right, 0.0)?;

        assert_eq!(operations.operations.len(), 0);

        let mut left = Rope::from_str(left);
        let new_right = operations.apply(&mut left)?;

        assert_eq!(new_right.to_string(), right);

        Ok(())
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

        test_merge_both_ways("hello world", "world !", "hi hello world", "hi world !");

        test_merge_both_ways(
            "both delete the same word",
            "both the same word",
            "both the same word",
            "both the same word",
        );

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
            .map(|text| text[0..50000].to_string())
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
        // println!("Original: {}", original);
        let mut original = Rope::from_str(original);

        let operations_1 =
            OperationSequence::try_from_string_diff(&original.to_string(), edit_1, 1.0).unwrap();
        let operations_2 =
            OperationSequence::try_from_string_diff(&original.to_string(), edit_2, 1.0).unwrap();
        // println!("Operations 1: {:?}", operations_1);
        // println!("Operations 2: {:?}", operations_2);

        assert_eq!(operations_1.apply(&mut original.clone()).unwrap(), edit_1);
        assert_eq!(operations_2.apply(&mut original.clone()).unwrap(), edit_2);

        let merged = operations_1.merge(&operations_2).unwrap();
        // println!("Merged: {:?}", merged);

        let result = merged.apply(&mut original).unwrap();
        result.to_string()
    }
}
