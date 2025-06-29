#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{CursorPosition, Operation, TextWithCursors};
use crate::{
    operation_transformation::utils::{
        cook_operations::cook_operations, elongate_operations::elongate_operations,
    },
    raw_operation::RawOperation,
    tokenizer::{Tokenizer, word_tokenizer::word_tokenizer},
    utils::{history::History, side::Side, string_builder::StringBuilder},
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
    operations: Vec<Operation<T>>,
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
    pub fn from_strings(original: &'a str, updated: TextWithCursors<'a>, side: Side) -> Self {
        Self::from_strings_with_tokenizer(original, updated, &word_tokenizer, side)
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
        side: Side,
    ) -> Self {
        let original_tokens = (tokenizer)(original);
        let updated_tokens = (tokenizer)(&updated.text);

        let diff: Vec<RawOperation<T>> = RawOperation::vec_from(&original_tokens, &updated_tokens);

        Self::new(
            original,
            cook_operations(elongate_operations(diff), side).collect(),
            updated.cursors,
        )
    }

    /// Create a new `EditedText` with the given operations.
    /// The operations must be in the order in which they are meant to be
    /// applied. The operations must not overlap.
    fn new(text: &'a str, operations: Vec<Operation<T>>, mut cursors: Vec<CursorPosition>) -> Self {
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

        let mut merged_cursors = Vec::with_capacity(self.cursors.len() + other.cursors.len());
        let mut left_cursors = self.cursors.into_iter().peekable();
        let mut right_cursors = other.cursors.into_iter().peekable();

        let mut merged_operations: Vec<Operation<T>> =
            Vec::with_capacity(self.operations.len() + other.operations.len());

        let mut left_iter = self.operations.into_iter();
        let mut right_iter = other.operations.into_iter();

        let mut maybe_left_op = left_iter.next();
        let mut maybe_right_op = right_iter.next();

        let mut seen_left_length: usize = 0;
        let mut seen_right_length: usize = 0;
        let mut merged_length: usize = 0;

        let mut last_left_op = None;
        let mut last_right_op = None;

        loop {
            let (side, operation, mut last_other_op) =
                match (maybe_left_op.clone(), maybe_right_op.clone()) {
                    (Some(left_op), Some(right_op)) => {
                        if left_op
                            .get_sort_key(seen_left_length)
                            .partial_cmp(&right_op.get_sort_key(seen_right_length))
                            == Some(std::cmp::Ordering::Less)
                        {
                            (Side::Left, left_op, last_right_op.clone())
                        } else {
                            (Side::Right, right_op, last_left_op.clone())
                        }
                    }

                    (Some(left_op), None) => (Side::Left, left_op, last_right_op.clone()),
                    (None, Some(right_op)) => (Side::Right, right_op, last_left_op.clone()),
                    (None, None) => break,
                };

            let is_advancing_operation = matches!(
                operation,
                Operation::Insert { .. } | Operation::Equal { .. }
            );

            let original_length = operation.len() as i64;
            let result = match side {
                Side::Left => {
                    let result = operation.merge_operations(&mut last_other_op);

                    if let ref op @ (Operation::Insert { .. } | Operation::Equal { .. }) = result {
                        let shift = merged_length as i64 - seen_left_length as i64
                            + op.len() as i64
                            - original_length;

                        while let Some(cursor) = left_cursors.next_if(|cursor| {
                            cursor.char_index <= seen_left_length + original_length as usize
                        }) {
                            merged_cursors.push(
                                cursor.with_index((cursor.char_index as i64 + shift) as usize),
                            );
                        }
                    }

                    if is_advancing_operation {
                        seen_left_length += original_length as usize;
                    }

                    maybe_left_op = left_iter.next();
                    last_left_op = Some(result.clone());

                    result
                }
                Side::Right => {
                    let result = operation.merge_operations(&mut last_other_op);

                    if let ref op @ (Operation::Insert { .. } | Operation::Equal { .. }) = result {
                        let shift = merged_length as i64 - seen_right_length as i64
                            + op.len() as i64
                            - original_length;

                        while let Some(cursor) = right_cursors.next_if(|cursor| {
                            cursor.char_index <= seen_right_length + original_length as usize
                        }) {
                            merged_cursors.push(
                                cursor.with_index((cursor.char_index as i64 + shift) as usize),
                            );
                        }
                    }

                    if is_advancing_operation {
                        seen_right_length += original_length as usize;
                    }

                    maybe_right_op = right_iter.next();
                    last_right_op = Some(result.clone());

                    result
                }
            };

            if result.len() == 0 {
                continue;
            }

            if is_advancing_operation {
                merged_length += result.len();
            }

            merged_operations.push(result);
        }

        for cursor in left_cursors.chain(right_cursors) {
            merged_cursors.push(cursor.with_index(merged_length));
        }

        Self::new(self.text, merged_operations, merged_cursors)
    }

    /// Apply the operations to the text and return the resulting text.
    #[must_use]
    pub fn apply(&self) -> String {
        let mut builder: StringBuilder<'_> = StringBuilder::new(self.text);

        for operation in &self.operations {
            builder = operation.apply(builder);
        }

        builder.take()
    }

    #[must_use]
    pub fn apply_with_history(&self) -> Vec<(History, String)> {
        let mut builder: StringBuilder<'_> = StringBuilder::new(self.text);

        let mut history = Vec::with_capacity(self.operations.len());

        for operation in &self.operations {
            builder = operation.apply(builder);

            match operation {
                Operation::Equal { .. } => history.push((History::Unchanged, builder.take())),
                Operation::Insert { side, .. } => match side {
                    Side::Left => history.push((History::AddedFromLeft, builder.take())),
                    Side::Right => history.push((History::AddedFromRight, builder.take())),
                },
                Operation::Delete {
                    deleted_character_count,
                    order,
                    side,
                    ..
                } => {
                    let deleted = self.text[*order..*order + *deleted_character_count].to_string();
                    match side {
                        Side::Left => history.push((History::RemovedFromLeft, deleted)),
                        Side::Right => history.push((History::RemovedFromRight, deleted)),
                    }
                }
            }
        }

        history
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_calculate_operations() {
        let left = "hello world! How are you?  Adam";
        let right = "Hello, my friend! How are you doing? Albert";

        let operations = EditedText::from_strings(left, right.into(), Side::Right);

        insta::assert_debug_snapshot!(operations);

        let new_right = operations.apply();
        assert_eq!(new_right.to_string(), right);
    }

    #[test]
    fn test_calculate_operations_with_no_diff() {
        let text = "hello world!";

        let operations = EditedText::from_strings(text, text.into(), Side::Right);

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

        let operations_1 = EditedText::from_strings(original, left.into(), Side::Left);
        let operations_2 = EditedText::from_strings(original, right.into(), Side::Right);

        let operations = operations_1.merge(operations_2);
        assert_eq!(operations.apply(), expected);
    }
}
