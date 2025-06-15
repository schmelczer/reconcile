#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{CursorPosition, Operation, TextWithCursors, ordered_operation::OrderedOperation};
use crate::{
    diffs::{myers::diff, raw_operation::RawOperation},
    operation_transformation::{
        merge_context::MergeContext,
        utils::{cook_operations::cook_operations, elongate_operations::elongate_operations},
    },
    tokenizer::{Tokenizer, word_tokenizer::word_tokenizer},
    utils::{side::Side, string_builder::StringBuilder},
};

/// A text document and a sequence of operations that can be applied to the text
/// document. `EditedText` supports merging two sequences of operations using
/// the principles of Operational Transformation.
///
/// It's mainly created through the `from_strings` method, then merged with
/// another `EditedText` derived from the same original text and then applied to
/// the original text to get the reconciled text of concurrent edits.
///
/// In addition to text and operations, it also keeps track of cursor positions
/// in the original text. The cursor positions are updated when the operations
/// are applied, so that the cursor positions can be used to restore the
/// cursor positions in the updated text.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EditedText<'a, T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    text: &'a str,
    operations: Vec<OrderedOperation<T>>,
    pub(crate) cursors: Vec<CursorPosition>,
}

impl<'a> EditedText<'a, String> {
    /// Create an `EditedText` from the given original (old) and updated (new)
    /// strings. The returned `EditedText` represents the changes from the
    /// original to the updated text. When the return value is applied to
    /// the original text, it will result in the updated text. The default
    /// word tokenizer is used to tokenize the text which splits the text on
    /// whitespaces.
    #[must_use]
    pub fn from_strings(original: &'a str, updated: TextWithCursors<'a>) -> Self {
        Self::from_strings_with_tokenizer(original, updated, &word_tokenizer)
    }
}

impl<'a, T> EditedText<'a, T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    /// Create an `EditedText` from the given original (old) and updated (new)
    /// strings. The returned `EditedText` represents the changes from the
    /// original to the updated text. When the return value is applied to
    /// the original text, it will result in the updated text. The tokenizer
    /// function is used to tokenize the text.
    pub fn from_strings_with_tokenizer(
        original: &'a str,
        updated: TextWithCursors<'a>,
        tokenizer: &Tokenizer<T>,
    ) -> Self {
        let original_tokens = (tokenizer)(original);
        let updated_tokens = (tokenizer)(&updated.text);

        let diff: Vec<RawOperation<T>> = diff(&original_tokens, &updated_tokens);

        Self::new(
            original,
            cook_operations(elongate_operations(diff)).collect(),
            updated.cursors,
        )
    }

    /// Create a new `EditedText` with the given operations.
    /// The operations must be in the order in which they are meant to be
    /// applied. The operations must not overlap.
    fn new(
        text: &'a str,
        operations: Vec<OrderedOperation<T>>,
        mut cursors: Vec<CursorPosition>,
    ) -> Self {
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

        cursors.sort_by_key(|cursor| cursor.char_index);

        Self {
            text,
            operations,
            cursors,
        }
    }

    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        debug_assert_eq!(
            self.text, other.text,
            "`EditedText`-s must be derived from the same text to be mergable"
        );

        let mut left_merge_context = MergeContext::default();
        let mut right_merge_context = MergeContext::default();

        let mut merged_cursors = Vec::with_capacity(self.cursors.len() + other.cursors.len());
        let mut left_cursors = self.cursors.into_iter().peekable();
        let mut right_cursors = other.cursors.into_iter().peekable();

        let mut merged_operations: Vec<OrderedOperation<T>> =
            Vec::with_capacity(self.operations.len() + other.operations.len());

        let mut left_iter = self.operations.into_iter();
        let mut right_iter = other.operations.into_iter();

        let mut maybe_left_op = left_iter.next();
        let mut maybe_right_op = right_iter.next();

        loop {
            let (side, OrderedOperation { operation, order }) =
                match (maybe_left_op.clone(), maybe_right_op.clone()) {
                    (Some(left_op), Some(right_op)) => {
                        if left_op < right_op {
                            (Side::Left, left_op)
                        } else {
                            (Side::Right, right_op)
                        }
                    }

                    (Some(left_op), None) => (Side::Left, left_op),
                    (None, Some(right_op)) => (Side::Right, right_op),
                    (None, None) => break,
                };

            if side == Side::Left {
                maybe_left_op = left_iter.next();
            } else {
                maybe_right_op = right_iter.next();
            }

            let original_start = operation.start_index() as i64;
            let original_end = operation.end_index();
            let original_length = operation.len() as i64;

            let result = match side {
                Side::Left => operation.merge_operations_with_context(
                    &mut right_merge_context,
                    &mut left_merge_context,
                ),
                Side::Right => operation.merge_operations_with_context(
                    &mut left_merge_context,
                    &mut right_merge_context,
                ),
            };

            if let Some(ref op @ (Operation::Insert { .. } | Operation::Equal { .. })) = result {
                let shift =
                    op.start_index() as i64 - original_start + op.len() as i64 - original_length;
                match side {
                    Side::Left => {
                        while let Some(cursor) =
                            left_cursors.next_if(|cursor| cursor.char_index <= original_end + 1)
                        {
                            merged_cursors.push(cursor.with_index(
                                (op.start_index() as i64).max(cursor.char_index as i64 + shift)
                                    as usize,
                            ));
                        }
                    }
                    Side::Right => {
                        while let Some(cursor) =
                            right_cursors.next_if(|cursor| cursor.char_index <= original_end + 1)
                        {
                            merged_cursors.push(cursor.with_index(
                                (op.start_index() as i64).max(cursor.char_index as i64 + shift)
                                    as usize,
                            ));
                        }
                    }
                }
            }

            merged_operations.extend(result.into_iter().map(|op| OrderedOperation {
                order,
                operation: op,
            }));
        }

        let last_index = merged_operations
            .iter()
            .filter(|operation| {
                matches!(
                    operation.operation,
                    Operation::Insert { .. } | Operation::Equal { .. }
                )
            })
            .next_back()
            .map_or(0, |op| op.operation.end_index());

        for cursor in left_cursors.chain(right_cursors) {
            merged_cursors.push(cursor.with_index(last_index));
        }

        Self::new(self.text, merged_operations, merged_cursors)
    }

    /// Apply the operations to the text and return the resulting text.
    #[must_use]
    pub fn apply(&self) -> String {
        let mut builder: StringBuilder<'_> = StringBuilder::new(self.text);

        for OrderedOperation { operation, .. } in &self.operations {
            builder = operation.apply(builder);
        }

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use insta::assert_debug_snapshot;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_calculate_operations() {
        let left = "hello world! How are you?  Adam";
        let right = "Hello, my friend! How are you doing? Albert";

        let operations = EditedText::from_strings(left, right.into());

        insta::assert_debug_snapshot!(operations);

        let new_right = operations.apply();
        assert_eq!(new_right.to_string(), right);
    }

    #[test]
    fn test_calculate_operations_with_no_diff() {
        let text = "hello world!";

        let operations = EditedText::from_strings(text, text.into());

        assert_debug_snapshot!(operations);

        let new_right = operations.apply();
        assert_eq!(new_right.to_string(), text);
    }

    #[test]
    fn test_calculate_operations_with_insert() {
        let original = "hello world! ...";
        let left = "Hello world! I'm Andras.";
        let right = "Hello world! How are you?";
        let expected = "Hello world! How are you? I'm Andras.";

        let operations_1 = EditedText::from_strings(original, left.into());
        let operations_2 = EditedText::from_strings(original, right.into());

        let operations = operations_1.merge(operations_2);
        assert_eq!(operations.apply(), expected);
    }
}
