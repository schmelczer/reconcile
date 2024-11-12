use super::{operation, Operation};
use crate::errors::SyncLibError;
use log::info;
use ropey::Rope;
use serde::{Deserialize, Serialize};
use similar::utils::diff_graphemes;
use similar::{utils::TextDiffRemapper, ChangeTag, TextDiff};
use similar::{Algorithm, DiffableStrRef};

#[derive(Debug)]
struct OperationWithTransformContext {
    operation: Option<Operation>,
    delete_state: Option<DeleteMergeState>,
    shift_change: i64,
}

#[derive(Debug, Clone)]
struct DeleteMergeState {
    start: u64,
    length: u64,
    is_same_side: bool,
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
        let operations = diff
            .ops()
            .iter()
            .flat_map(move |x| remapper.iter_slices(x))
            .map(|(tag, text)| match tag {
                ChangeTag::Equal => {
                    index += text.chars().count();
                    None
                }
                ChangeTag::Insert => {
                    let result = Some(Operation::new(tag, index as u64, text));
                    index += text.chars().count();
                    result
                }
                ChangeTag::Delete => Some(Operation::new(tag, index as u64, text)),
            })
            .flat_map(Option::into_iter)
            .collect::<Vec<_>>();

        Ok(Self::new(operations))
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

        let mut left_delete_state: Option<DeleteMergeState> = None;
        let mut right_delete_state: Option<DeleteMergeState> = None;

        let mut left_cursor_offset: i64 = 0;
        let mut right_cursor_offset: i64 = 0;
        let mut left_index: usize = 0;
        let mut right_index: usize = 0;

        loop {
            let left_op = self.operations.get(left_index);
            let right_op = other.operations.get(right_index);
            println!("");
            println!(
                "{} <> {}",
                left_op.cloned().unwrap_or_default(),
                right_op.cloned().unwrap_or_default()
            );
            println!(
                "cursor_offset: {} <> {}",
                left_cursor_offset, right_cursor_offset
            );
            println!("{:?} <> {:?}", left_delete_state, right_delete_state);

            match (left_op, right_op, left_op.cmp(&right_op)) {
                (Some(left_op), None, _)
                | (Some(left_op), Some(_), std::cmp::Ordering::Less | std::cmp::Ordering::Equal) => {
                    println!("Left op: {:?}", left_op);

                    let context = Self::merge_operation_with_state(
                        left_op,
                        right_delete_state.clone(),
                        left_cursor_offset as i64,
                    )?;
                    println!("Context: {:?}", context);
                    if let Some(op) = context.operation {
                        merged_operations.push(op);
                    }

                    if let Some(DeleteMergeState {
                        is_same_side: false,
                        ..
                    }) = context.delete_state
                    {
                        left_delete_state = context.delete_state;
                    } else {
                        right_delete_state = context.delete_state;
                    }

                    right_cursor_offset += context.shift_change;
                    left_index += 1;
                }
                (None, Some(right_op), _)
                | (Some(_), Some(right_op), std::cmp::Ordering::Greater) => {
                    println!("Right op: {:?}", right_op);
                    let context = Self::merge_operation_with_state(
                        right_op,
                        left_delete_state.clone(),
                        right_cursor_offset as i64,
                    )?;
                    println!("Context: {:?}", context);
                    if let Some(op) = context.operation {
                        merged_operations.push(op);
                    }
                    if let Some(DeleteMergeState {
                        is_same_side: false,
                        ..
                    }) = context.delete_state
                    {
                        right_delete_state = context.delete_state;
                    } else {
                        left_delete_state = context.delete_state;
                    }

                    left_cursor_offset += context.shift_change;
                    right_index += 1;
                }
                (None, None, _) => {
                    break;
                }
            };
        }

        Ok(Self::new(merged_operations))
    }

    fn merge_operation_with_state(
        operation: &Operation,
        state: Option<DeleteMergeState>,
        shift: i64,
    ) -> Result<OperationWithTransformContext, SyncLibError> {
        Ok(match (operation, state) {
            (Operation::Insert { text, .. }, None) => OperationWithTransformContext {
                operation: Some(operation.with_shifted_index(shift)?),
                delete_state: None,
                shift_change: text.chars().count() as i64,
            },

            (
                Operation::Delete {
                    index,
                    deleted_character_count,
                },
                None,
            ) => OperationWithTransformContext {
                operation: Some(operation.with_shifted_index(shift)?),
                delete_state: Some(DeleteMergeState {
                    start: (*index as i64 + shift).try_into().map_err(|_| {
                        SyncLibError::OperationShiftingError("Failed to shift index".to_string())
                    })?,
                    length: *deleted_character_count,
                    is_same_side: false,
                }),
                shift_change: 0,
            },

            (Operation::Insert { index, text }, Some(state)) => {
                if (state.start..state.start + state.length).contains(index) {
                    let len = text.chars().count() as u64;
                    OperationWithTransformContext {
                        operation: Some(operation.with_index(state.start)),
                        delete_state: Some(DeleteMergeState {
                            start: state.start + len,
                            length: state.length.saturating_sub(len),
                            is_same_side: true,
                        }),
                        shift_change: len as i64,
                    }
                } else {
                    let len = text.chars().count() as i64;
                    OperationWithTransformContext {
                        operation: Some(operation.with_shifted_index(shift - state.length as i64)?),
                        delete_state: None,
                        shift_change: len - (state.length as i64),
                    }
                }
            }

            (
                Operation::Delete {
                    index,
                    deleted_character_count,
                },
                Some(state),
            ) => {
                let translated_index = *index as i64 + shift;
                if (state.start as i64..state.start as i64 + state.length as i64)
                    .contains(&translated_index)
                    && (state.start as i64..state.start as i64 + state.length as i64)
                        .contains(&(translated_index as i64 + *deleted_character_count as i64 - 1))
                {
                    OperationWithTransformContext {
                        operation: None,
                        delete_state: Some(state),
                        shift_change: 0,
                    }
                } else if (state.start as i64..state.start as i64 + state.length as i64)
                    .contains(&translated_index)
                {
                    let overlap =
                        (state.start + state.length).saturating_add_signed(translated_index);
                    OperationWithTransformContext {
                        operation: Some(Operation::Delete {
                            index: state.start + state.length,
                            deleted_character_count: deleted_character_count - overlap,
                        }),
                        delete_state: Some(DeleteMergeState {
                            start: state.start + state.length,
                            length: deleted_character_count - overlap,
                            is_same_side: false,
                        }),
                        shift_change: -(overlap as i64),
                    }
                } else {
                    OperationWithTransformContext {
                        operation: Some(operation.with_shifted_index(shift - state.length as i64)?),
                        delete_state: Some(DeleteMergeState {
                            start: ((*index as i64 + shift) - state.length as i64)
                                .try_into()
                                .map_err(|_| {
                                    SyncLibError::OperationShiftingError(
                                        "Failed to shift index".to_string(),
                                    )
                                })?,
                            length: *deleted_character_count,
                            is_same_side: false,
                        }),
                        shift_change: -(state.length as i64),
                    }
                }
            }
        })
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
        // test_merge(
        //     "hello world",
        //     "hi, world",
        //     "hello my friend!",
        //     "hi, my friend!",
        // );

        // test_merge("hello world", "world !", "hi hello world", "hi world !");

        test_merge("a b", "c d", "a b c d", "c d c d")
    }

    fn test_merge(original: &str, edit_1: &str, edit_2: &str, expected: &str) {
        let mut original = Rope::from_str(original);

        let operations_1 =
            OperationSequence::try_from_string_diff(&original.to_string(), edit_1, 1.0).unwrap();
        let operations_2 =
            OperationSequence::try_from_string_diff(&original.to_string(), edit_2, 1.0).unwrap();
        let merged = operations_1.merge(&operations_2).unwrap();
        println!("Operations 1: {:?}", operations_1);
        println!("Operations 2: {:?}", operations_2);
        println!("Merged: {:?}", merged);

        let result = merged.apply(&mut original).unwrap();

        assert_eq!(result, expected);
    }
}
