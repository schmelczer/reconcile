use core::fmt::{Debug, Display};
use std::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::merge_context::MergeContext;
use crate::{
    Token,
    utils::{
        find_longest_prefix_contained_within::find_longest_prefix_contained_within,
        string_builder::StringBuilder,
    },
};

/// Represents a change that can be applied on a `StringBuilder`.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq)]
pub enum Operation<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    Equal {
        index: usize,
        length: usize,

        #[cfg(debug_assertions)]
        text: Option<String>,
    },

    Insert {
        index: usize,
        text: Vec<Token<T>>,
    },

    Delete {
        index: usize,
        deleted_character_count: usize,

        #[cfg(debug_assertions)]
        deleted_text: Option<String>,
    },
}

impl<T> Operation<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    /// Creates an equal operation with the given index.
    /// This operation is used to indicate that the text at the given index
    /// is unchanged.
    pub fn create_equal(index: usize, length: usize) -> Option<Self> {
        if length == 0 {
            return None;
        }

        Some(Operation::Equal {
            index,
            length,

            #[cfg(debug_assertions)]
            text: None,
        })
    }

    pub fn create_equal_with_text(index: usize, text: String) -> Option<Self> {
        if text.is_empty() {
            return None;
        }

        Some(Operation::Equal {
            index,
            length: text.chars().count(),

            #[cfg(debug_assertions)]
            text: Some(text),
        })
    }

    /// Creates an insert operation with the given index and text.
    /// If the text is empty (meaning that the operation would be a no-op),
    /// returns None.
    pub fn create_insert(index: usize, text: Vec<Token<T>>) -> Option<Self> {
        if text.is_empty() {
            return None;
        }

        Some(Operation::Insert { index, text })
    }

    /// Creates a delete operation with the given index and number of
    /// to-be-deleted characters. If the operation would delete 0 (meaning
    /// that the operation would be a no-op), returns None.
    pub fn create_delete(index: usize, deleted_character_count: usize) -> Option<Self> {
        if deleted_character_count == 0 {
            return None;
        }

        Some(Operation::Delete {
            index,
            deleted_character_count,

            #[cfg(debug_assertions)]
            deleted_text: None,
        })
    }

    pub fn create_delete_with_text(index: usize, text: String) -> Option<Self> {
        if text.is_empty() {
            return None;
        }

        Some(Operation::Delete {
            index,
            deleted_character_count: text.chars().count(),

            #[cfg(debug_assertions)]
            deleted_text: Some(text),
        })
    }

    /// Applies the operation to the given `StringBuilder`, returning the
    /// modified `StringBuilder`.
    ///
    /// When compiled in debug mode, panics if a delete operation is attempted
    /// on a range of text that does not match the text to be deleted.
    pub fn apply<'a>(&self, mut builder: StringBuilder<'a>) -> StringBuilder<'a> {
        match self {
            Operation::Equal {
                #[cfg(debug_assertions)]
                text,
                ..
            } => {
                #[cfg(debug_assertions)]
                debug_assert!(
                    text.as_ref()
                        .is_none_or(|text| builder.get_slice(self.range()) == *text),
                    "Text which is supposed to be equal does not match the text in the range"
                );

                return builder;
            }
            Operation::Insert { text, .. } => builder.insert(
                self.start_index(),
                &text.iter().map(Token::original).collect::<String>(),
            ),
            Operation::Delete {
                #[cfg(debug_assertions)]
                deleted_text,
                ..
            } => {
                #[cfg(debug_assertions)]
                debug_assert!(
                    deleted_text
                        .as_ref()
                        .is_none_or(|text| builder.get_slice(self.range()) == *text),
                    "Text to delete does not match the text in the range"
                );

                builder.delete(self.range());
            }
        }

        builder
    }

    /// Returns the index of the first character that the operation affects.
    pub fn start_index(&self) -> usize {
        match self {
            Operation::Equal { index, .. }
            | Operation::Insert { index, .. }
            | Operation::Delete { index, .. } => *index,
        }
    }

    /// Returns the index of the last character that the operation affects.
    pub fn end_index(&self) -> usize {
        debug_assert!(
            self.len() > 0,
            " len() must be greater than 0 because operations must be non-empty"
        );
        self.start_index() + self.len() - 1
    }

    /// Returns the range of indices of characters that the operation affects.
    #[allow(clippy::range_plus_one)]
    pub fn range(&self) -> Range<usize> { self.start_index()..self.end_index() + 1 }

    /// Returns the number of affected characters. It is always greater than 0
    /// because empty operations cannot be created.
    pub fn len(&self) -> usize {
        match self {
            Operation::Equal { length, .. } => *length,
            Operation::Insert { text, .. } => text.iter().map(Token::get_original_length).sum(),
            Operation::Delete {
                deleted_character_count,
                ..
            } => *deleted_character_count,
        }
    }

    /// Creates a new operation with the same type and text but with the given
    /// index.
    pub fn with_index(self, index: usize) -> Self {
        match self {
            Operation::Equal {
                length,

                #[cfg(debug_assertions)]
                text,
                ..
            } => Operation::Equal {
                index,
                length,

                #[cfg(debug_assertions)]
                text,
            },
            Operation::Insert { text, .. } => Operation::Insert { index, text },
            Operation::Delete {
                deleted_character_count,

                #[cfg(debug_assertions)]
                deleted_text,
                ..
            } => Operation::Delete {
                index,
                deleted_character_count,

                #[cfg(debug_assertions)]
                deleted_text,
            },
        }
    }

    /// Creates a new operation with the same type and text but with the index
    /// shifted by the given offset. The offset can be negative but the
    /// resulting index must be non-negative.
    ///
    /// # Panics
    ///
    /// In debug mode, panics if the resulting index is negative.
    pub fn with_shifted_index(self, offset: i64) -> Self {
        let index = self.start_index() as i64 + offset;
        debug_assert!(index >= 0, "Shifted index must be non-negative");

        self.with_index(index as usize)
    }

    /// Merges the operation with the given context, producing a new operation
    /// and updating the context. This implements a comples FSM that handles
    /// the merging of operations in a way that is consistent with the text.
    /// The contexts are updated in-place.
    pub fn merge_operations_with_context(
        self,
        affecting_context: &mut MergeContext<T>,
        produced_context: &mut MergeContext<T>,
    ) -> Option<Operation<T>> {
        affecting_context.consume_last_operation_if_it_is_too_behind(self.start_index() as i64);
        let operation = self.with_shifted_index(affecting_context.shift);

        match (operation, affecting_context.last_operation()) {
            (operation @ Operation::Insert { .. }, None | Some(Operation::Equal { .. })) => {
                produced_context.shift += operation.len() as i64;
                produced_context.consume_and_replace_last_operation(Some(operation.clone()));
                Some(operation)
            }

            (
                Operation::Insert { text, index },
                Some(Operation::Insert {
                    text: previous_inserted_text,
                    ..
                }),
            ) => {
                // In case the current insert's prefix appears in the previously inserted text,
                // we can trim the current insert to only include the non-overlapping part.
                // This way, we don't end up duplicating text.
                let offset_in_tokens =
                    find_longest_prefix_contained_within(previous_inserted_text, &text);
                let offset_in_length = text
                    .iter()
                    .take(offset_in_tokens)
                    .map(Token::get_original_length)
                    .sum::<usize>();
                let trimmed_operation =
                    Operation::create_insert(index, text[offset_in_tokens..].to_vec());

                affecting_context.shift -= offset_in_length as i64;
                produced_context.shift += trimmed_operation
                    .as_ref()
                    .map(Operation::len)
                    .unwrap_or_default() as i64;
                produced_context.consume_and_replace_last_operation(trimmed_operation.clone());

                trimmed_operation
            }

            (
                operation @ Operation::Delete { .. },
                None | Some(Operation::Insert { .. }) | Some(Operation::Equal { .. }),
            ) => {
                produced_context.consume_and_replace_last_operation(Some(operation.clone()));
                Some(operation)
            }

            (
                operation @ Operation::Insert { .. },
                Some(last_delete @ Operation::Delete { .. }),
            ) => {
                produced_context.shift += operation.len() as i64;

                debug_assert!(
                    last_delete.range().contains(&operation.start_index()),
                    "There is a last delete ({last_delete}) but the operation ({operation}) is \
                     not contained in it"
                );

                let difference = operation.start_index() as i64 - last_delete.start_index() as i64;

                let moved_operation = operation.with_index(last_delete.start_index());

                affecting_context.replace_last_operation(Operation::create_delete(
                    moved_operation.end_index() + 1,
                    (last_delete.len() as i64 - difference) as usize,
                ));
                affecting_context.shift -= difference;

                produced_context.consume_and_replace_last_operation(Some(moved_operation.clone()));

                Some(moved_operation)
            }

            (
                operation @ Operation::Delete { .. },
                Some(last_delete @ Operation::Delete { .. }),
            ) => {
                debug_assert!(
                    last_delete.range().contains(&operation.start_index()),
                    "There is a last delete ({last_delete}) but the operation ({operation}) is \
                     not contained in it"
                );

                let difference = operation.start_index() as i64 - last_delete.start_index() as i64;

                let updated_delete = Operation::create_delete(
                    last_delete.start_index(),
                    0.max(operation.end_index() as i64 - last_delete.end_index() as i64) as usize,
                );

                affecting_context.replace_last_operation(Operation::create_delete(
                    last_delete.start_index(),
                    0.max(last_delete.end_index() as i64 - operation.end_index() as i64) as usize,
                ));
                affecting_context.shift -= difference;

                produced_context.consume_and_replace_last_operation(updated_delete.clone());

                updated_delete
            }
            (
                ref operation @ Operation::Equal {
                    length,
                    #[cfg(debug_assertions)]
                    ref text,
                    ..
                },
                Some(last_delete @ Operation::Delete { .. }),
            ) => {
                debug_assert!(
                    last_delete.range().contains(&operation.start_index()),
                    "There is a last delete ({last_delete}) but the operation ({operation}) is \
                     not contained in it"
                );

                let overlap = (length as i64)
                    .min(last_delete.end_index() as i64 - operation.start_index() as i64 + 1);

                if cfg!(debug_assertions) && text.is_some() {
                    Operation::create_equal_with_text(
                        operation.end_index().min(last_delete.end_index()),
                        text.clone()
                            .unwrap()
                            .chars()
                            .skip(overlap as usize)
                            .collect::<String>(),
                    )
                } else {
                    Operation::create_equal(
                        operation.end_index().min(last_delete.end_index()),
                        (length as i64 - overlap) as usize,
                    )
                }
            }
            (operation @ Operation::Equal { .. }, _) => Some(operation),
        }
    }
}

impl<T> Display for Operation<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Operation::Equal {
                index,
                length,

                #[cfg(debug_assertions)]
                text,
            } => {
                #[cfg(debug_assertions)]
                write!(
                    f,
                    "<equal {} from index {}>",
                    text.as_ref()
                        .map(|text| format!("'{text}'"))
                        .unwrap_or(format!("{length} characters")),
                    index
                )?;

                #[cfg(not(debug_assertions))]
                write!(f, "<equal {length} from index {index}>")?;

                Ok(())
            }
            Operation::Insert { index, text } => {
                write!(
                    f,
                    "<insert '{}' from index {}>",
                    text.iter().map(Token::original).collect::<String>(),
                    index
                )
            }
            Operation::Delete {
                index,
                deleted_character_count,

                #[cfg(debug_assertions)]
                deleted_text,
            } => {
                #[cfg(debug_assertions)]
                write!(
                    f,
                    "<delete {} from index {}>",
                    deleted_text
                        .as_ref()
                        .map(|text| format!("'{text}'"))
                        .unwrap_or(format!("{deleted_character_count} characters")),
                    index
                )?;

                #[cfg(not(debug_assertions))]
                write!(
                    f,
                    "<delete {deleted_character_count} characters from index {index}>",
                )?;

                Ok(())
            }
        }
    }
}

impl<T> Debug for Operation<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "{self}") }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    #[should_panic(expected = "Shifted index must be non-negative")]
    fn test_shifting_error() {
        insta::assert_debug_snapshot!(
            Operation::create_insert(1, vec!["hi".into()])
                .unwrap()
                .with_shifted_index(-2)
        );
    }

    #[test]
    fn test_apply_delete_with_create() {
        let builder = StringBuilder::new("hello world");
        let operation = Operation::<()>::create_delete_with_text(5, " world".to_owned()).unwrap();

        assert_eq!(operation.apply(builder).build(), "hello");
    }

    #[test]
    fn test_apply_insert() {
        let builder = StringBuilder::new("hello");
        let operation = Operation::create_insert(5, vec![" my friend".into()]).unwrap();

        assert_eq!(operation.apply(builder).build(), "hello my friend");
    }
}
