use super::{operation, Operation};
use crate::errors::SyncLibError;
use log::info;
use ropey::Rope;
use serde::{de, Deserialize, Serialize};
use similar::utils::diff_graphemes;
use similar::{utils::TextDiffRemapper, ChangeTag, TextDiff};
use similar::{Algorithm, DiffableStrRef};

#[derive(Debug, Clone, Default)]
struct MergeContext {
    last_delete: Option<Operation>,
    shift: i64,
}

enum Source {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationSequence {
    operations: Vec<Operation>,
}

impl OperationSequence {
    pub fn new(mut operations: Vec<Operation>) -> Self {
        operations.sort();

        Self { operations }
    }

    pub fn try_from_string_diff(
        left: &str,
        right: &str,
        diff_ratio_threshold: f32,
    ) -> Result<Self, SyncLibError> {
        let diff = TextDiff::configure()
            .algorithm(Algorithm::Myers)
            .diff_words(left, right);

        let diff_ratio = 1.0 - diff.ratio();
        if diff_ratio > diff_ratio_threshold {
            return Err(SyncLibError::DiffTooLarge {
                diff_ratio,
                diff_ratio_limit: diff_ratio_threshold as f32,
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
                    None
                }
                ChangeTag::Insert => {
                    let result = Some(Operation::create(tag, index as i64, text));
                    index += text.chars().count();
                    result
                }
                ChangeTag::Delete => Some(Operation::create(tag, index as i64, text)),
            })
            .flat_map(Option::into_iter)
            .collect::<Result<Vec<_>, SyncLibError>>()
            .map(Self::new)
    }

    pub fn apply<'a>(&self, rope_text: &'a mut Rope) -> Result<&'a mut Rope, SyncLibError> {
        for operation in &self.operations {
            println!("Applying operation: {:?}", operation);
            operation.apply(rope_text)?;
            println!("Text after operation: {}", rope_text);
        }

        Ok(rope_text)
    }

    pub fn merge(&self, other: &Self) -> Result<Self, SyncLibError> {
        let mut merged_operations =
            Vec::with_capacity(self.operations.len() + other.operations.len());

        let mut left_delete_context = MergeContext::default();
        let mut right_delete_context = MergeContext::default();

        let mut left_index: usize = 0;
        let mut right_index: usize = 0;

        loop {
            let left_op = self.operations.get(left_index);
            let right_op = other.operations.get(right_index);
            println!("");
            println!("{:#?} <> {:#?}", left_op.cloned(), right_op.cloned());

            println!("{:?} <> {:?}", left_delete_context, right_delete_context);

            match (left_op, right_op, left_op.cmp(&right_op)) {
                (Some(left_op), None, _)
                | (Some(left_op), Some(_), std::cmp::Ordering::Less | std::cmp::Ordering::Equal) => {
                    println!("Left op: {:?}", left_op);

                    if let Some(op) = Self::merge_operations_with_context(
                        left_op,
                        &mut right_delete_context,
                        &mut left_delete_context,
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
                        &mut left_delete_context,
                        &mut right_delete_context,
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
            println!("{:?} <> {:?}", left_delete_context, right_delete_context);
        }

        Ok(Self::new(merged_operations))
    }

    fn merge_operations_with_context(
        operation: &Operation,
        affecting_context: &mut MergeContext,
        produced_context: &mut MergeContext,
    ) -> Result<Option<Operation>, SyncLibError> {
        Ok(match (operation, affecting_context.last_delete.clone()) {
            (Operation::Insert { .. }, None) => {
                produced_context.shift += operation.len();
                Some(operation.with_shifted_index(affecting_context.shift))
            }

            (Operation::Delete { .. }, None) => {
                let operation = Some(operation.with_shifted_index(affecting_context.shift));
                Self::replace_delete_in_produced_context(produced_context, operation.clone());
                operation
            }

            (Operation::Insert { .. }, Some(last_delete)) => {
                produced_context.shift += operation.len();

                if last_delete
                    .range()
                    .contains(&(&operation.start_index() + affecting_context.shift))
                {
                    affecting_context.last_delete = Some(Operation::create_delete(
                        last_delete.start_index() + operation.len(),
                        0.max(last_delete.len() - operation.len()),
                    )?);

                    Some(operation.with_index(last_delete.start_index()))
                } else {
                    Self::pick_up_dangling_delete_from_affecting_context(
                        affecting_context,
                        last_delete,
                    );
                    Some(operation.with_shifted_index(affecting_context.shift))
                }
            }

            (Operation::Delete { .. }, Some(last_delete)) => {
                let shifted_operation = operation.with_shifted_index(affecting_context.shift);

                if last_delete
                    .range()
                    .contains(&shifted_operation.start_index())
                    && last_delete.range().contains(&shifted_operation.end_index())
                {
                    affecting_context.shift -=
                        shifted_operation.start_index() - last_delete.start_index();
                    affecting_context.last_delete = Some(Operation::create_delete(
                        shifted_operation.end_index() + 1,
                        last_delete.end_index() - shifted_operation.end_index(),
                    )?);

                    None
                } else if last_delete
                    .range()
                    .contains(&shifted_operation.start_index())
                {
                    let overlap = last_delete.end_index()
                        - (operation.start_index() + affecting_context.shift)
                        + 1;
                    affecting_context.last_delete = None;
                    affecting_context.shift -= last_delete.len() - overlap;

                    let operation = Some(Operation::create_delete(
                        last_delete.start_index(),
                        operation.len() - overlap,
                    )?);
                    Self::replace_delete_in_produced_context(produced_context, operation.clone());
                    operation
                } else {
                    Self::pick_up_dangling_delete_from_affecting_context(
                        affecting_context,
                        last_delete,
                    );

                    let operation = Some(operation.with_shifted_index(affecting_context.shift));
                    Self::replace_delete_in_produced_context(produced_context, operation.clone());
                    operation
                }
            }
        })
    }

    fn replace_delete_in_produced_context(
        produced_context: &mut MergeContext,
        delete: Option<Operation>,
    ) {
        if let Some(produced_last_delete) = produced_context.last_delete.take() {
            produced_context.shift -= produced_last_delete.len();
        }

        produced_context.last_delete = delete;
    }

    fn pick_up_dangling_delete_from_affecting_context(
        affecting_context: &mut MergeContext,
        last_delete: Operation,
    ) {
        affecting_context.shift -= last_delete.len();
        affecting_context.last_delete = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::test;

    use super::*;

    #[test]
    fn test_calculate_operations() -> Result<(), SyncLibError> {
        let left = "hello world! How are you?  Adam";
        let right = "Hello, my friend! How are you doing? Albert";

        let operations = OperationSequence::try_from_string_diff(left, right, 0.6)?;

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
        test_merge_both_ways(
            "hello world",
            "hi, world",
            "hello my friend!",
            "hi, my friend!",
        );

        test_merge_both_ways("hello world", "world !", "hi hello world", "hi world !");

        test_merge_both_ways("a b", "c d", "a b c d", "c d c d");

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
            "both same word",
        );

        test_merge_both_ways(
            "long text with one big delete and many small",
            "long small",
            "long with big and small",
            "long small",
        );
    }

    fn test_merge_both_ways(original: &str, edit_1: &str, edit_2: &str, expected: &str) {
        test_merge(original, edit_1, edit_2, expected);
        test_merge(original, edit_2, edit_1, expected);
    }

    fn test_merge(original: &str, edit_1: &str, edit_2: &str, expected: &str) {
        let mut original = Rope::from_str(original);

        let operations_1 =
            OperationSequence::try_from_string_diff(&original.to_string(), edit_1, 1.0).unwrap();
        let operations_2 =
            OperationSequence::try_from_string_diff(&original.to_string(), edit_2, 1.0).unwrap();
        println!("Operations 1: {:?}", operations_1);
        println!("Operations 2: {:?}", operations_2);

        assert_eq!(operations_1.apply(&mut original.clone()).unwrap(), edit_1);
        assert_eq!(operations_2.apply(&mut original.clone()).unwrap(), edit_2);

        let merged = operations_1.merge(&operations_2).unwrap();
        println!("Merged: {:?}", merged);

        let result = merged.apply(&mut original).unwrap();
        assert_eq!(result, expected);
    }
}
