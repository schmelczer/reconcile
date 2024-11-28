use ropey::Rope;
use std::fmt::Display;

use crate::{errors::SyncLibError, Token};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::merge_context::MergeContext;

/// Represents a change that can be applied to a text document.
/// Operation is tied to a ropey::Rope and is mainly expected to be
/// created by EditedText.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Operation<T>
where
    T: PartialEq + Clone,
{
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
    T: PartialEq + Clone,
{
    /// Creates an insert operation with the given index and text.
    /// If the text is empty (meaning that the operation would be a no-op), returns None.
    pub fn create_insert(index: usize, text: Vec<Token<T>>) -> Option<Self> {
        if text.is_empty() {
            return None;
        }

        Some(Operation::Insert { index, text })
    }

    /// Creates a delete operation with the given index and number of to-be-deleted characters.
    /// If the operation would delete 0 (meaning that the operation would be a no-op), returns None.
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

    /// Tries to apply the operation to the given ropey::Rope text, returning the modified text.
    ///
    /// # Errors
    ///
    /// Returns a SyncLibError::OperationApplicationError if the operation cannot be applied.
    ///
    /// # Panics
    ///
    /// When compiled in debug mode, panics if a delete operation is attempted on a range
    /// of text that does not match the text to be deleted.
    pub fn apply<'a>(&self, rope_text: &'a mut Rope) -> Result<&'a mut Rope, SyncLibError> {
        match self {
            Operation::Insert { text, .. } => rope_text
                .try_insert(
                    self.start_index(),
                    &text
                        .iter()
                        .map(|token| token.original())
                        .collect::<String>(),
                )
                .map_err(|err| {
                    SyncLibError::OperationApplicationError(format!(
                        "Failed to insert text: {}",
                        err
                    ))
                }),
            Operation::Delete {
                #[cfg(debug_assertions)]
                deleted_text,
                ..
            } => {
                debug_assert!(
                    rope_text.get_slice(self.range()).is_some(),
                    "Failed to get slice of text to delete"
                );

                if let Some(text) = deleted_text {
                    debug_assert_eq!(
                        rope_text.get_slice(self.range()).unwrap().to_string(),
                        *text,
                        "Text to delete does not match the text in the rope"
                    );
                }

                rope_text.try_remove(self.range()).map_err(|err| {
                    SyncLibError::OperationApplicationError(format!(
                        "Failed to remove text: {}",
                        err
                    ))
                })
            }
        }?;

        Ok(rope_text)
    }

    /// Returns the index of the first character that the operation affects.
    pub fn start_index(&self) -> usize {
        match self {
            Operation::Insert { index, .. } => *index,
            Operation::Delete { index, .. } => *index,
        }
    }

    /// Returns the index of the last character that the operation affects.
    pub fn end_index(&self) -> usize {
        // len() must be greater than 0 because operations must be non-empty
        self.start_index() + self.len() - 1
    }

    /// Returns the range of indices of characters that the operation affects, inclusive.
    pub fn range(&self) -> std::ops::RangeInclusive<usize> {
        self.start_index()..=self.end_index()
    }

    /// Returns the number of affected characters. It is always greater than 0 because empty operations cannot be created.
    pub fn len(&self) -> usize {
        match self {
            Operation::Insert { text, .. } => {
                text.iter().map(|token| token.get_original_length()).sum()
            }
            Operation::Delete {
                deleted_character_count,
                ..
            } => *deleted_character_count,
        }
    }

    /// Clones the operation while updating the index.
    pub fn with_index(self, index: usize) -> Self {
        match self {
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

    /// Clones the operation while shifting the index by the given offset.
    /// The offset can be negative but the resulting index must be non-negative.
    ///
    /// # Panics
    ///
    /// In debug mode, panics if the resulting index is negative.
    pub fn with_shifted_index(self, offset: i64) -> Self {
        let index = self.start_index() as i64 + offset;
        debug_assert!(index >= 0, "Shifted index must be non-negative");

        self.with_index(index as usize)
    }

    /// Merges the operation with the given context, producing a new operation and updating the context.
    /// This implements a comples FSM that handles the merging of operations in a way that is consistent with the text.
    /// The contexts are updated in-place.
    pub fn merge_operations_with_context(
        self,
        affecting_context: &mut MergeContext<T>,
        produced_context: &mut MergeContext<T>,
    ) -> Option<Operation<T>> {
        affecting_context.consume_last_operation_if_it_is_too_behind(&self);

        let operation = self.with_shifted_index(affecting_context.shift);

        match (operation, affecting_context.last_operation().clone()) {
            (operation @ Operation::Insert { .. }, None) => {
                produced_context.shift += operation.len() as i64;
                Some(operation)
            }

            (operation, Some(last_insert @ Operation::Insert { .. })) => {
                produced_context.shift += operation.len() as i64;
                Some(operation)
            }

            // We can never delete inside an insert
            (operation @ Operation::Delete { .. }, None) => {
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
                        "There is a last delete ({last_delete}) but the operation ({operation}) is not contained in it"
                    );

                let difference = operation.start_index() as i64 - last_delete.start_index() as i64;

                let moved_operation = operation.with_index(last_delete.start_index());

                affecting_context.replace_last_operation(Operation::create_delete(
                    moved_operation.end_index() + 1,
                    (last_delete.len() as i64 - difference) as usize,
                ));
                affecting_context.shift -= difference;

                Some(moved_operation)
            }

            (
                operation @ Operation::Delete { .. },
                Some(last_delete @ Operation::Delete { .. }),
            ) => {
                debug_assert!(
                        last_delete.range().contains(&operation.start_index()),
                        "There is a last delete ({last_delete}) but the operation ({operation}) is not contained in it"
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
        }
    }
}

impl<T> Display for Operation<T>
where
    T: PartialEq + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Insert { index, text } => {
                write!(
                    f,
                    "<insert '{}' from index {}>",
                    text.iter()
                        .map(|token| token.original())
                        .collect::<String>(),
                    index
                )
            }
            Operation::Delete {
                index,
                deleted_character_count,

                #[cfg(debug_assertions)]
                deleted_text,
            } => {
                if cfg!(debug_assertions) && deleted_text.is_some() {
                    write!(
                        f,
                        "<delete '{}' from index {}>",
                        deleted_text.as_ref().unwrap_or(&"<unknown>".to_string()),
                        index
                    )
                } else {
                    write!(
                        f,
                        "<delete {} characters () from index {}>",
                        deleted_character_count, index
                    )
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    #[should_panic]
    fn test_shifting_error() {
        insta::assert_debug_snapshot!(Operation::create_insert(1, vec!["hi".into()])
            .unwrap()
            .with_shifted_index(-2));
    }

    #[test]
    fn test_apply_delete_with_create() -> Result<(), SyncLibError> {
        let mut rope = Rope::from_str("hello world");
        let operation = Operation::<()>::create_delete_with_text(5, " world".to_string()).unwrap();

        operation.apply(&mut rope)?;

        assert_eq!(rope.to_string(), "hello");

        Ok(())
    }

    #[test]
    fn test_apply_insert() -> Result<(), SyncLibError> {
        let mut rope = Rope::from_str("hello");
        let operation = Operation::create_insert(5, vec![" my friend".into()]).unwrap();

        operation.apply(&mut rope)?;

        assert_eq!(rope.to_string(), "hello my friend");

        Ok(())
    }
}
