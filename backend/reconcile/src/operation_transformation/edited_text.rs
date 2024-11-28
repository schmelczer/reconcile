use super::Operation;
use crate::diffs::raw_operation::RawOperation;
use crate::errors::SyncLibError;
use crate::operation_transformation::merge_context::MergeContext;
use crate::tokenizer::word_tokenizer::word_tokenizer;
use crate::tokenizer::Tokenizer;
use crate::utils::ordered_operation::OrderedOperation;
use crate::utils::side::Side;
use crate::{diffs::myers::diff, utils::merge_iters::MergeSorted};
use ropey::Rope;
use std::iter;

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
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EditedText<'a, T>
where
    T: PartialEq + Clone,
{
    text: &'a str,
    operations: Vec<OrderedOperation<T>>,
}

impl<'a> EditedText<'a, String> {
    /// Create an EditedText from the given original (old) and updated (new) strings.
    /// The returned EditedText represents the changes from the original to the updated text.
    /// When the return value is applied to the original text, it will result in the updated text.
    /// The default word tokenizer is used to tokenize the text which splits the text on whitespaces.
    pub fn from_strings(original: &'a str, updated: &str) -> Self {
        Self::from_strings_with_tokenizer(original, updated, &word_tokenizer)
    }
}

impl<'a, T> EditedText<'a, T>
where
    T: PartialEq + Clone,
{
    /// Create an EditedText from the given original (old) and updated (new) strings.
    /// The returned EditedText represents the changes from the original to the updated text.
    /// When the return value is applied to the original text, it will result in the updated text.
    /// The tokenizer function is used to tokenize the text.
    pub fn from_strings_with_tokenizer(
        original: &'a str,
        updated: &str,
        tokenizer: &Tokenizer<T>,
    ) -> Self {
        let original_tokens = (tokenizer)(original);
        let updated_tokens = (tokenizer)(updated);

        let diff: Vec<RawOperation<T>> = diff(&original_tokens, &updated_tokens);

        Self::new(
            original,
            // Self::cook_operations(diff),
            Self::cook_operations(Self::elongate_operations(diff)).collect(),
        )
    }

    // Turn raw operations into ordered operations while keeping track of old & new indexes.
    fn cook_operations<I>(raw_operations: I) -> impl Iterator<Item = OrderedOperation<T>>
    where
        I: IntoIterator<Item = RawOperation<T>>,
    {
        let mut new_index = 0; // this is the start index of the operation on the new text
        let mut order = 0; // this is the start index of the operation on the original text

        raw_operations.into_iter().flat_map(move |raw_operation| {
            let length = raw_operation.original_text_length();

            let operation = match raw_operation {
                RawOperation::Equal(..) => {
                    new_index += length;
                    order += length;

                    None
                }
                RawOperation::Insert(tokens) => {
                    let op = Operation::create_insert(new_index, tokens)
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
    }

    fn elongate_operations<I>(raw_operations: I) -> Vec<RawOperation<T>>
    where
        I: IntoIterator<Item = RawOperation<T>>,
    {
        let mut maybe_previous_insert: Option<RawOperation<T>> = None;
        let mut maybe_previous_delete: Option<RawOperation<T>> = None;

        let mut result: Vec<RawOperation<T>> = raw_operations
            .into_iter()
            .flat_map(|next| match next {
                RawOperation::Insert(..) => {
                    if let Some(prev) = maybe_previous_insert.take() {
                        maybe_previous_insert = prev.extend(next);
                    } else {
                        maybe_previous_insert = Some(next);
                    }

                    Box::new(iter::empty()) as Box<dyn Iterator<Item = RawOperation<T>>>
                }
                RawOperation::Delete(..) => {
                    if let Some(prev) = maybe_previous_delete.take() {
                        maybe_previous_delete = prev.extend(next);
                    } else {
                        maybe_previous_delete = Some(next);
                    }

                    Box::new(iter::empty()) as Box<dyn Iterator<Item = RawOperation<T>>>
                }
                RawOperation::Equal(..) => Box::new(
                    maybe_previous_insert
                        .take()
                        .into_iter()
                        .chain(maybe_previous_delete.take())
                        .chain(iter::once(next)),
                )
                    as Box<dyn Iterator<Item = RawOperation<T>>>,
            })
            .collect();

        if let Some(prev) = maybe_previous_insert {
            result.push(prev);
        }

        if let Some(prev) = maybe_previous_delete {
            result.push(prev);
        }

        result
    }

    /// Create a new EditedText with the given operations.
    /// The operations must be in the order in which they are meant to be applied.
    /// The operations must not overlap.
    fn new(text: &'a str, operations: Vec<OrderedOperation<T>>) -> Self {
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

    #[test]
    fn test_calculate_operations_with_insert() {
        let original = "hello world! ...";
        let left = "hello world! I'm Andras.";
        let right = "Hello world! How are you?";
        let expected = "Hello world! I'm Andras.How are you?";

        let operations_1 = EditedText::from_strings(original, left);
        println!("{:#?}", operations_1);
        let operations_2 = EditedText::from_strings(original, right);
        println!("{:#?}", operations_2);

        let operations = operations_1.merge(operations_2);

        assert_eq!(operations.apply().unwrap(), expected);
    }
}
