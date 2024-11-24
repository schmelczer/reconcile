use super::Operation;
use crate::diffs::raw_operation::RawOperation;
use crate::errors::SyncLibError;
use crate::operation_transformation::merge_context::MergeContext;
use crate::tokenizer::token::Token;
use crate::utils::ordered_operation::OrderedOperation;
use crate::utils::side::Side;
use crate::{diffs::myers::diff, utils::merge_iters::MergeSorted};
use ropey::Rope;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A sequence of operations that can be applied to a text document.
/// EditedText supports merging two sequences of operations using the
/// principle of Operational Transformation.
///
/// It's mainly created through the from_strings method, then merged with another
/// EditedText derived from the same original text and then applied to the original text
/// to get the reconciled text of concurrent edits.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct EditedText<'a> {
    text: &'a str,
    operations: Vec<OrderedOperation>,
}

impl<'a> EditedText<'a> {
    /// Create an EditedText from the given original (old) and updated (new) strings.
    /// The returned EditedText represents the changes from the original to the updated text.
    /// When the return value is applied to the original text, it will result in the updated text.
    pub fn from_strings(original: &'a str, updated: &str) -> Self {
        let original_tokens = Token::tokenize(original);
        let updated_tokens = Token::tokenize(updated);

        let diff: Vec<RawOperation> = diff(&original_tokens, &updated_tokens);

        Self::new(
            original,
            Self::elongate_operations(Self::cook_operations(diff)),
        )
    }

    // Turn raw operations into ordered operations while keeping track of old & new indexes.
    fn cook_operations(raw_operations: Vec<RawOperation>) -> Vec<OrderedOperation> {
        let mut new_index = 0; // this is the start index of the operation on the new text
        let mut order = 0; // this is the start index of the operation on the original text

        raw_operations
            .into_iter()
            .flat_map(|raw_operation| {
                let length = raw_operation.original_text_length();

                let operation = match raw_operation {
                    RawOperation::Equal(..) => {
                        new_index += length;
                        order += length;

                        None
                    }
                    RawOperation::Insert(..) => {
                        let op =
                            Operation::create_insert(new_index, raw_operation.get_original_text())
                                .map(|operation| OrderedOperation { order, operation });

                        new_index += length;

                        op
                    }
                    RawOperation::Delete(..) => {
                        let op = if cfg!(debug_assertions) {
                            Operation::create_delete_with_text(
                                new_index,
                                raw_operation.get_original_text(),
                            )
                        } else {
                            Operation::create_delete(new_index, length)
                        }
                        .map(|operation| OrderedOperation { order, operation });

                        order += length;

                        op
                    }
                };

                operation.into_iter()
            })
            .collect()
    }

    // TODO: shift ops befor compacting
    fn elongate_operations(operations: Vec<OrderedOperation>) -> Vec<OrderedOperation> {
        let mut maybe_previous: Option<OrderedOperation> = None;

        let mut result: Vec<OrderedOperation> = operations
            .into_iter()
            .flat_map(|next| {
                if let Some(previous) = maybe_previous.take() {
                    match (previous, next) {
                        (
                            previous @ OrderedOperation {
                                operation: Operation::Insert { .. },
                                ..
                            },
                            next @ OrderedOperation {
                                operation: Operation::Insert { .. },
                                ..
                            },
                        ) if previous.operation.end_index() + 1 == next.operation.start_index() => {
                            maybe_previous = Some(OrderedOperation {
                                order: previous.order,
                                operation: previous.operation.extend(&next.operation),
                            });
                            None
                        }
                        (
                            previous @ OrderedOperation {
                                operation: Operation::Delete { .. },
                                ..
                            },
                            next @ OrderedOperation {
                                operation: Operation::Delete { .. },
                                ..
                            },
                        ) if previous.operation.start_index() == next.operation.start_index() => {
                            maybe_previous = Some(OrderedOperation {
                                order: previous.order,
                                operation: previous.operation.extend(&next.operation),
                            });
                            None
                        }
                        (previous, next) => {
                            maybe_previous = Some(next);
                            Some(previous)
                        }
                    }
                } else {
                    maybe_previous = Some(next.clone());
                    None
                }
                .into_iter()
            })
            .collect();

        if let Some(prev) = maybe_previous {
            result.push(prev);
        }

        result
    }

    /// Create a new EditedText with the given operations.
    /// The operations must be in the order in which they are meant to be applied.
    /// The operations must not overlap.
    fn new(text: &'a str, operations: Vec<OrderedOperation>) -> Self {
        operations
            .iter()
            .zip(operations.iter().skip(1))
            .for_each(|(previous, next)| {
                debug_assert!(
                    previous.operation.start_index() <= next.operation.start_index(),
                    "{} must not come before {} yet it does",
                    previous.operation,
                    next.operation
                );
            });

        Self { text, operations }
    }

    pub fn merge(self, other: Self) -> Self {
        debug_assert_eq!(
            self.text, other.text,
            "EditedTexts must be derived from the same text to be mergable"
        );

        let mut left_merge_context = MergeContext::default();
        let mut right_merge_context = MergeContext::default();

        Self::new(
            self.text,
            self.operations
                .into_iter()
                .map(|op| (op, Side::Left))
                .merge_sorted_by_key(
                    other.operations.into_iter().map(|op| (op, Side::Right)),
                    |(operation, _)| operation.order,
                )
                .flat_map(|(OrderedOperation { order, operation }, side)| {
                    match side {
                        Side::Left => operation.merge_operations_with_context(
                            &mut right_merge_context,
                            &mut left_merge_context,
                        ),
                        Side::Right => operation.merge_operations_with_context(
                            &mut left_merge_context,
                            &mut right_merge_context,
                        ),
                    }
                    .map(|operation| OrderedOperation { order, operation })
                    .into_iter()
                })
                .collect(),
        )
    }

    /// Apply the operations to the text and return the resulting text.
    ///
    /// # Errors
    ///
    /// Returns an SyncLibError::OperationError if the operations cannot be applied to the text.
    pub fn apply(&self) -> Result<String, SyncLibError> {
        let mut text = Rope::from_str(self.text);
        self.operations
            .iter()
            .try_fold(
                &mut text,
                |rope_text, OrderedOperation { operation, .. }| operation.apply(rope_text),
            )
            .map(|rope| rope.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_calculate_operations() {
        let left = "hello world! How are you?  Adam";
        let right = "Hello, my friend! How are you doing? Albert";

        let operations = EditedText::from_strings(left, right);

        insta::assert_debug_snapshot!(operations);

        let new_right = operations.apply().unwrap();

        assert_eq!(new_right.to_string(), right);
    }

    #[test]
    fn test_calculate_operations_with_no_diff() {
        let text = "hello world!";

        let operations = EditedText::from_strings(text, text);

        assert_eq!(operations.operations.len(), 0);

        let new_right = operations.apply().unwrap();

        assert_eq!(new_right.to_string(), text);
    }

    // #[test]
    // fn test_calculate_operations_with_insert() {
    //     let original = "hello world! ...";
    //     let left = "Hello world! How are you?";
    //     let right = "hello world! I'm Andras.";
    //     let expected = "Hello world! I'm Andras. How are you?";

    //     let operations_1 = EditedText::from_strings(original, left);
    //     println!("{:#?}", operations_1);
    //     let operations_2 = EditedText::from_strings(original, right);
    //     println!("{:#?}", operations_2);

    //     let operations = operations_1.merge(operations_2);

    //     assert_eq!(operations.apply().unwrap(), expected);
    // }
}
