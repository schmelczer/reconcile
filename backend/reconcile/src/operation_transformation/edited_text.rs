use core::iter;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{CursorPosition, Operation, TextWithCursors, ordered_operation::OrderedOperation};
use crate::{
    diffs::{myers::diff, raw_operation::RawOperation},
    operation_transformation::merge_context::MergeContext,
    tokenizer::{Tokenizer, word_tokenizer::word_tokenizer},
    utils::{merge_iters::MergeSorted as _, side::Side, string_builder::StringBuilder},
};

/// A sequence of operations that can be applied to a text document.
/// `EditedText` supports merging two sequences of operations using the
/// principle of Operational Transformation.
///
/// It's mainly created through the `from_strings` method, then merged with
/// another `EditedText` derived from the same original text and then applied to
/// the original text to get the reconciled text of concurrent edits.
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
            Self::cook_operations(Self::elongate_operations(diff)).collect(),
            updated.cursors,
        )
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
                RawOperation::Insert(..) => match maybe_previous_insert.take() {
                    Some(prev) if prev.is_right_joinable() && next.is_left_joinable() => {
                        maybe_previous_insert = prev.extend(next);
                        Box::new(iter::empty()) as Box<dyn Iterator<Item = RawOperation<T>>>
                    }
                    prev => {
                        maybe_previous_insert = Some(next);
                        Box::new(prev.into_iter())
                    }
                },
                RawOperation::Delete(..) => match maybe_previous_delete.take() {
                    Some(prev) if prev.is_right_joinable() && next.is_left_joinable() => {
                        maybe_previous_delete = prev.extend(next);
                        Box::new(iter::empty()) as Box<dyn Iterator<Item = RawOperation<T>>>
                    }
                    prev => {
                        maybe_previous_delete = Some(next);
                        Box::new(prev.into_iter())
                    }
                },
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

    // Turn raw operations into ordered operations while keeping track of old & new
    // indexes.
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
        let mut left_cursors = self.cursors.iter().peekable();
        let mut right_cursors = other.cursors.iter().peekable();

        let merged_operations = self
            .operations
            .into_iter()
            // The current text is always the left; the other operation is the right side.
            .map(|op| (op, Side::Left))
            .merge_sorted_by_key(
                other.operations.into_iter().map(|op| (op, Side::Right)),
                |(operation, _)| {
                    (
                        operation.order,
                        // Operations on the left and right must come in the same order so that
                        // inserts can be merged with other inserts and deletes with deletes.
                        usize::from(matches!(operation.operation, Operation::Delete { .. })),
                        operation.operation.start_index(),
                        // Make sure that the ordering is deterministic regardless which text
                        // is left or right.
                        match &operation.operation {
                            Operation::Insert { text, .. } => text
                                .iter()
                                .map(super::super::tokenizer::token::Token::original)
                                .collect::<String>(),
                            Operation::Delete {
                                deleted_character_count,
                                ..
                            } => deleted_character_count.to_string(),
                        },
                    )
                },
            )
            .flat_map(|(OrderedOperation { order, operation }, side)| {
                match side {
                    Side::Left => {
                        while let Some(cursor) = left_cursors
                            .next_if(|cursor| cursor.char_index <= operation.start_index())
                        {
                            right_merge_context.consume_last_operation_if_it_is_too_behind(
                                cursor.char_index as i64,
                            );
                            merged_cursors.push(cursor.apply_merge_context(&right_merge_context));
                        }

                        while let Some(cursor) = right_cursors.next_if(|cursor| {
                            cursor.char_index as i64
                                <= operation.start_index() as i64 + right_merge_context.shift
                                    - left_merge_context.shift
                        }) {
                            left_merge_context.consume_last_operation_if_it_is_too_behind(
                                cursor.char_index as i64,
                            );
                            merged_cursors.push(cursor.apply_merge_context(&left_merge_context));
                        }

                        operation.merge_operations_with_context(
                            &mut right_merge_context,
                            &mut left_merge_context,
                        )
                    }
                    Side::Right => {
                        while let Some(cursor) = right_cursors
                            .next_if(|cursor| cursor.char_index <= operation.start_index())
                        {
                            left_merge_context.consume_last_operation_if_it_is_too_behind(
                                cursor.char_index as i64,
                            );
                            merged_cursors.push(cursor.apply_merge_context(&left_merge_context));
                        }

                        while let Some(cursor) = left_cursors.next_if(|cursor| {
                            cursor.char_index as i64
                                <= operation.start_index() as i64 + left_merge_context.shift
                                    - right_merge_context.shift
                        }) {
                            right_merge_context.consume_last_operation_if_it_is_too_behind(
                                cursor.char_index as i64,
                            );
                            merged_cursors.push(cursor.apply_merge_context(&right_merge_context));
                        }

                        operation.merge_operations_with_context(
                            &mut left_merge_context,
                            &mut right_merge_context,
                        )
                    }
                }
                .map(|operation| OrderedOperation { order, operation })
                .into_iter()
            })
            .collect();

        for cursor in left_cursors {
            right_merge_context
                .consume_last_operation_if_it_is_too_behind(cursor.char_index as i64);
            merged_cursors.push(cursor.apply_merge_context(&right_merge_context));
        }

        for cursor in right_cursors {
            left_merge_context.consume_last_operation_if_it_is_too_behind(cursor.char_index as i64);
            merged_cursors.push(cursor.apply_merge_context(&left_merge_context));
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

        assert_eq!(operations.operations.len(), 0);

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
