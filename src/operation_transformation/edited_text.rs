use std::{fmt::Debug, vec};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    BuiltinTokenizer, CursorPosition, TextWithCursors,
    operation_transformation::{
        DiffError, Operation,
        utils::{cook_operations::cook_operations, elongate_operations::elongate_operations},
    },
    raw_operation::RawOperation,
    tokenizer::Tokenizer,
    types::{
        history::History, number_or_text::NumberOrText, side::Side,
        span_with_history::SpanWithHistory,
    },
    utils::string_builder::StringBuilder,
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
    T: PartialEq + Clone + Debug,
{
    text: &'a str,
    operations: Vec<Operation<T>>,
    operation_sides: Vec<Side>,
    cursors: Vec<CursorPosition>,
}

impl<'a> EditedText<'a, String> {
    /// Create an `EditedText` from the given original (old) and updated (new)
    /// strings. The returned `EditedText` represents the changes from the
    /// original to the updated text. When the return value is applied to
    /// the original text, it will result in the updated text. The default
    /// word tokenizer is used to tokenize the text which splits the text on
    /// whitespaces.
    #[must_use]
    pub fn from_strings(original: &'a str, updated: &TextWithCursors) -> Self {
        Self::from_strings_with_tokenizer(original, updated, &*BuiltinTokenizer::Word)
    }
}

impl<'a, T> EditedText<'a, T>
where
    T: PartialEq + Clone + Debug,
{
    /// Create an `EditedText` from the given original (old) and updated (new)
    /// strings. The returned `EditedText` represents the changes from the
    /// original to the updated text. When the return value is applied to
    /// the original text, it will result in the updated text. The tokenizer
    /// function is used to tokenize the text.
    pub fn from_strings_with_tokenizer(
        original: &'a str,
        updated: &TextWithCursors,
        tokenizer: &Tokenizer<T>,
    ) -> Self {
        let original_tokens = (tokenizer)(original);
        let updated_tokens = (tokenizer)(&updated.text());

        let diff: Vec<RawOperation<T>> = RawOperation::vec_from(&original_tokens, &updated_tokens);
        let operations: Vec<Operation<T>> = cook_operations(elongate_operations(diff)).collect();
        let operation_count = operations.len();

        Self::new(
            original,
            operations,
            vec![Side::Left; operation_count],
            updated.cursors(),
        )
    }

    /// Create a new `EditedText` with the given operations.
    /// The operations must be in the order in which they are meant to be
    /// applied. The operations must not overlap.
    fn new(
        text: &'a str,
        operations: Vec<Operation<T>>,
        operation_sides: Vec<Side>,
        mut cursors: Vec<CursorPosition>,
    ) -> Self {
        cursors.sort_by_key(|cursor| cursor.char_index);

        Self {
            text,
            operations,
            operation_sides,
            cursors,
        }
    }

    /// Merge two `EditedText` instances. The two instances must be derived
    /// from the same original text. The operations are merged using the
    /// principles of Operational Transformation. The cursors are updated
    /// accordingly to reflect the changes made by the merged operations.
    ///
    /// # Panics
    ///
    /// Panics if there's an integer overflow (in i64) when calculating new
    /// cursor positions.
    #[must_use]
    #[allow(clippy::too_many_lines)]
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
        let mut merged_operation_sides: Vec<Side> =
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

            let original_length = operation.len();
            let (side, result) = match side {
                Side::Left => {
                    let result = operation.merge_operations(&mut last_other_op);

                    if let ref op @ (Operation::Insert { .. } | Operation::Equal { .. }) = result {
                        let merged_length_signed = isize::try_from(merged_length)
                            .expect("merged_length must fit in isize");
                        let seen_left_length_signed = isize::try_from(seen_left_length)
                            .expect("seen_left_length must fit in isize");
                        let op_len_signed =
                            isize::try_from(op.len()).expect("op.len() must fit in isize");
                        let original_length_signed = isize::try_from(original_length)
                            .expect("original_length must fit in isize");

                        let shift = merged_length_signed - seen_left_length_signed + op_len_signed
                            - original_length_signed;

                        while let Some(cursor) = left_cursors.next_if(|cursor| {
                            cursor.char_index <= seen_left_length + original_length
                        }) {
                            merged_cursors.push(
                                cursor.with_index(cursor.char_index.saturating_add_signed(shift)),
                            );
                        }
                    }

                    if is_advancing_operation {
                        seen_left_length += original_length;
                    }

                    maybe_left_op = left_iter.next();
                    last_left_op = Some(result.clone());

                    (Side::Left, result)
                }
                Side::Right => {
                    let result = operation.merge_operations(&mut last_other_op);

                    if let ref op @ (Operation::Insert { .. } | Operation::Equal { .. }) = result {
                        let merged_length_signed = isize::try_from(merged_length)
                            .expect("merged_length must fit in isize");
                        let seen_right_length_signed = isize::try_from(seen_right_length)
                            .expect("seen_right_length must fit in isize");
                        let op_len_signed =
                            isize::try_from(op.len()).expect("op.len() must fit in isize");
                        let original_length_signed = isize::try_from(original_length)
                            .expect("original_length must fit in isize");

                        let shift = merged_length_signed - seen_right_length_signed + op_len_signed
                            - original_length_signed;

                        while let Some(cursor) = right_cursors.next_if(|cursor| {
                            cursor.char_index <= seen_right_length + original_length
                        }) {
                            merged_cursors.push(
                                cursor.with_index(cursor.char_index.saturating_add_signed(shift)),
                            );
                        }
                    }

                    if is_advancing_operation {
                        seen_right_length += original_length;
                    }

                    maybe_right_op = right_iter.next();
                    last_right_op = Some(result.clone());

                    (Side::Right, result)
                }
            };

            if result.len() == 0 {
                continue;
            }

            if is_advancing_operation {
                merged_length += result.len();
            }

            merged_operations.push(result);
            merged_operation_sides.push(side);
        }

        for cursor in left_cursors.chain(right_cursors) {
            merged_cursors.push(cursor.with_index(merged_length));
        }

        debug_assert_eq!(merged_operations.len(), merged_operation_sides.len());

        Self::new(
            self.text,
            merged_operations,
            merged_operation_sides,
            merged_cursors,
        )
    }

    /// Apply the operations to the text and return the resulting text.
    #[must_use]
    pub fn apply(&self) -> TextWithCursors {
        let mut builder: StringBuilder<'_> = StringBuilder::new(self.text);

        for operation in &self.operations {
            builder = operation.apply(builder);
        }

        TextWithCursors::new(builder.take(), self.cursors.clone())
    }

    /// Apply the operations to the text and return the resulting text in chunks
    /// together with the provenance describing where each chunk came from.
    ///
    /// The result includes deleted spans as well.
    ///
    /// ```
    ///  use reconcile_text::{History, SpanWithHistory, BuiltinTokenizer, reconcile};
    ///
    ///  let parent = "Merging text is hard!";
    ///  let left = "Merging text is easy!"; // Changed "hard" to "easy"
    ///  let right = "With reconcile, merging documents is hard!"; // Added prefix and changed word
    ///
    ///  let result = reconcile(
    ///      parent,
    ///      &left.into(),
    ///      &right.into(),
    ///      &*BuiltinTokenizer::Word,
    ///  );
    ///
    ///  assert_eq!(
    ///      result.apply_with_history(),
    ///      vec![
    ///          SpanWithHistory::new("Merging text".to_string(), History::RemovedFromRight,),
    ///          SpanWithHistory::new(
    ///              "With reconcile, merging documents".to_string(),
    ///              History::AddedFromRight,
    ///          ),
    ///          SpanWithHistory::new(" ".to_string(), History::Unchanged,),
    ///          SpanWithHistory::new("is".to_string(), History::Unchanged,),
    ///          SpanWithHistory::new(" hard!".to_string(), History::RemovedFromLeft,),
    ///          SpanWithHistory::new(" easy!".to_string(), History::AddedFromLeft,),
    ///      ]
    ///  );
    /// ```
    #[must_use]
    pub fn apply_with_history(&self) -> Vec<SpanWithHistory> {
        let mut builder: StringBuilder<'_> = StringBuilder::new(self.text);

        let mut history = Vec::with_capacity(self.operations.len());

        for (operation, side) in self.operations.iter().zip(self.operation_sides.iter()) {
            builder = operation.apply(builder);

            match operation {
                Operation::Equal { .. } => {
                    history.push(SpanWithHistory::new(builder.take(), History::Unchanged));
                }
                Operation::Insert { .. } => match side {
                    Side::Left => {
                        history.push(SpanWithHistory::new(builder.take(), History::AddedFromLeft));
                    }
                    Side::Right => history.push(SpanWithHistory::new(
                        builder.take(),
                        History::AddedFromRight,
                    )),
                },
                Operation::Delete {
                    deleted_character_count,
                    order,
                    ..
                } => {
                    let deleted = self.text[*order..*order + *deleted_character_count].to_string();
                    match side {
                        Side::Left => {
                            history.push(SpanWithHistory::new(deleted, History::RemovedFromLeft));
                        }
                        Side::Right => {
                            history.push(SpanWithHistory::new(deleted, History::RemovedFromRight));
                        }
                    }
                }
            }
        }

        history
    }

    /// Convert the `EditedText` into a terse representation ready for
    /// serialization. The result omits cursor positions and the original text.
    /// This is useful for sending text diffs over the network if there's a
    /// clear consensus on the original text.
    ///
    /// Inserts are represented as strings, deletes as negative integers,
    /// and equal spans as positive integers.
    ///
    /// # Panics
    ///
    /// Panics if there's an integer overflow in i64.
    #[must_use]
    pub fn to_diff(&self) -> Vec<NumberOrText> {
        let mut result: Vec<NumberOrText> = Vec::with_capacity(self.operations.len());
        let mut previous_equal: Option<usize> = None;

        for operation in &self.operations {
            match operation {
                Operation::Equal { length, .. } => {
                    if let Some(prev_length) = previous_equal {
                        previous_equal = Some(prev_length + *length);
                    } else {
                        previous_equal = Some(*length);
                    }
                }

                Operation::Insert { text, .. } => {
                    if let Some(prev_length) = previous_equal {
                        result.push(NumberOrText::Number(
                            i64::try_from(prev_length).expect("prev_length must fit in i64"),
                        ));
                        previous_equal = None;
                    }

                    let text: String = text
                        .iter()
                        .map(super::super::tokenizer::token::Token::original)
                        .collect();
                    result.push(NumberOrText::Text(text));
                }

                Operation::Delete {
                    deleted_character_count,
                    ..
                } => {
                    if let Some(prev_length) = previous_equal {
                        result.push(NumberOrText::Number(
                            i64::try_from(prev_length).expect("prev_length must fit in i64"),
                        ));
                        previous_equal = None;
                    }

                    let count = i64::try_from(*deleted_character_count)
                        .expect("deleted_character_count must fit in i64");
                    result.push(NumberOrText::Number(-count));
                }
            }
        }

        if let Some(prev_length) = previous_equal {
            result.push(NumberOrText::Number(
                i64::try_from(prev_length).expect("prev_length must fit in i64"),
            ));
        }

        result
    }

    /// Deserialize an `EditedText` from a change list and the original text.
    ///
    /// # Errors
    ///
    /// Returns `DiffError::LengthExceedsOriginal` if the diff references a
    /// range that exceeds the original text length.
    ///
    /// # Panics
    ///
    /// Panics if there's an integer overflow in i64.
    pub fn from_diff(
        original_text: &'a str,
        diff: Vec<NumberOrText>,
        tokenizer: &Tokenizer<T>,
    ) -> Result<EditedText<'a, T>, DiffError> {
        let mut operations: Vec<Operation<T>> = Vec::with_capacity(diff.len());
        let mut order = 0;

        for item in diff {
            match item {
                NumberOrText::Number(length) => {
                    if length >= 0 {
                        let length = usize::try_from(length).expect("length must fit in usize");

                        // Validate that the range doesn't exceed the original text
                        let text_length = original_text.chars().count();
                        if order + length > text_length {
                            return Err(DiffError::LengthExceedsOriginal {
                                position: order,
                                requested: length,
                                available: text_length.saturating_sub(order),
                            });
                        }

                        let original_characters: String =
                            original_text.chars().skip(order).take(length).collect();

                        let original_tokens = tokenizer(&original_characters);
                        for token in original_tokens {
                            operations
                                .push(Operation::create_equal(order, token.get_original_length()));
                            order += token.get_original_length();
                        }
                    } else {
                        let length =
                            usize::try_from(-length).expect("negative length must fit in usize");

                        // Validate that the delete range doesn't exceed the original text
                        let text_length = original_text.chars().count();
                        if order + length > text_length {
                            return Err(DiffError::LengthExceedsOriginal {
                                position: order,
                                requested: length,
                                available: text_length.saturating_sub(order),
                            });
                        }

                        operations.push(Operation::create_delete(order, length));
                        order += length;
                    }
                }
                NumberOrText::Text(text) => {
                    let tokens = tokenizer(&text);
                    operations.push(Operation::create_insert(order, tokens));
                }
            }
        }

        let operation_count = operations.len();
        Ok(EditedText::new(
            original_text,
            operations,
            vec![Side::Left; operation_count],
            vec![],
        ))
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

        let operations = EditedText::from_strings(left, &right.into());

        insta::assert_debug_snapshot!(operations);

        let new_right = operations.apply();
        assert_eq!(new_right.text(), right);
    }

    #[test]
    fn test_calculate_operations_with_no_diff() {
        let text = "hello world!";

        let operations = EditedText::from_strings(text, &text.into());

        assert_debug_snapshot!(operations);

        let new_right = operations.apply();
        assert_eq!(new_right.text(), text);
    }

    #[test]
    fn test_calculate_operations_with_insert() {
        let original = "hello world! ...";
        let left = "Hello world! I'm Andras.";
        let right = "Hello world! How are you?";
        let expected = "Hello world! How are you? I'm Andras.";

        let operations_1 = EditedText::from_strings(original, &left.into());
        let operations_2 = EditedText::from_strings(original, &right.into());

        let operations = operations_1.merge(operations_2);
        assert_eq!(operations.apply().text(), expected);
    }

    #[test]
    fn test_from_diff_length_exceeds_original() {
        let result = EditedText::from_diff(
            "hello",
            vec![
                10.into(), // too large equal span - should error
                " world".into(),
            ],
            &*BuiltinTokenizer::Word,
        );

        assert!(result.is_err());
        match result {
            Err(DiffError::LengthExceedsOriginal {
                position,
                requested,
                available,
            }) => {
                assert_eq!(position, 0);
                assert_eq!(requested, 10);
                assert_eq!(available, 5);
            }
            _ => panic!("Expected LengthExceedsOriginal error"),
        }
    }

    #[test]
    fn test_from_diff_valid() {
        let edited_text = EditedText::from_diff(
            "hello",
            vec![
                5.into(), // exact length
                " world".into(),
            ],
            &*BuiltinTokenizer::Word,
        )
        .unwrap();

        let content = edited_text.apply().text();

        assert_eq!(content, "hello world");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_changes_deserialisation() {
        let original = "Merging text is hard!";
        let changes = "Merging text is easy with reconcile!";
        let result = EditedText::from_strings(original, &changes.into());
        let serialized = serde_yaml::to_string(&result.to_diff()).unwrap();

        let expected = concat!("- 15\n", "- -6\n", "- ' easy with reconcile!'\n",);
        assert_eq!(serialized, expected);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_changes_serialization() {
        let original = "The quick brown fox jumps over the lazy dog.";
        let updated = "The quick red fox jumped over the very lazy dog!";

        let edited_text = EditedText::from_strings(original, &updated.into());

        let changes = edited_text.to_diff();
        let deserialized_edited_text =
            EditedText::from_diff(original, changes, &*BuiltinTokenizer::Word).unwrap();

        assert_eq!(deserialized_edited_text.apply().text(), updated);
    }
}
