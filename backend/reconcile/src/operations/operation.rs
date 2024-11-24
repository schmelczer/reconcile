use ropey::Rope;
use std::fmt::Display;

use crate::errors::SyncLibError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents a change that can be applied to a text document.
/// Operation is tied to a ropey::Rope and is mainly expected to be
/// created by OperationSequence.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operation {
    Insert {
        index: usize,
        text: String,
    },

    Delete {
        index: usize,
        deleted_character_count: usize,

        #[cfg(debug_assertions)]
        deleted_text: Option<String>,
    },
}

impl Operation {
    /// Creates an insert operation with the given index and text.
    /// If the text is empty (meaning that the operation would be a no-op), returns None.
    pub fn create_insert(index: usize, text: String) -> Option<Self> {
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
    pub fn apply<'a>(&self, rope_text: &'a mut Rope) -> Result<&'a mut Rope, SyncLibError> {
        match self {
            Operation::Insert { text, .. } => rope_text
                .try_insert(self.start_index(), text)
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
            Operation::Insert { text, .. } => text.chars().count(),
            Operation::Delete {
                deleted_character_count,
                ..
            } => *deleted_character_count,
        }
    }

    /// The operation cannot be empty.
    pub fn is_empty(&self) -> bool {
        debug_assert!(self.len() > 0, "Operation cannot be empty");
        false
    }

    /// Clones the operation while updating the index.
    pub fn with_index(&self, index: usize) -> Self {
        match self {
            Operation::Insert { text, .. } => Operation::Insert {
                index,
                text: text.clone(),
            },
            Operation::Delete {
                deleted_character_count,

                #[cfg(debug_assertions)]
                deleted_text,
                ..
            } => Operation::Delete {
                index,
                deleted_character_count: *deleted_character_count,

                #[cfg(debug_assertions)]
                deleted_text: deleted_text.clone(),
            },
        }
    }

    /// Clones the operation while shifting the index by the given offset.
    /// The offset can be negative but the resulting index must be non-negative.
    pub fn with_shifted_index(&self, offset: i64) -> Result<Self, SyncLibError> {
        let index = self.start_index() as i64 + offset;
        let non_negative_index = index.try_into().map_err(|_| {
            SyncLibError::NegativeOperationIndexError(format!(
                "Index {} is negative but operations must have a non-negative index",
                index
            ))
        })?;

        Ok(self.with_index(non_negative_index))
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Insert { index, text } => {
                write!(f, "<insert '{}' from index {}>", text, index)
            }
            Operation::Delete {
                index,
                deleted_character_count,

                #[cfg(debug_assertions)]
                deleted_text,
            } => {
                if cfg!(debug_assertions) {
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
    fn test_shifting_error() {
        insta::assert_debug_snapshot!(Operation::create_insert(1, "hi".to_string())
            .unwrap()
            .with_shifted_index(-2));
    }

    #[test]
    fn test_apply_delete_with_create() -> Result<(), SyncLibError> {
        let mut rope = Rope::from_str("hello world");
        let operation = Operation::create_delete_with_text(5, " world".to_string()).unwrap();

        operation.apply(&mut rope)?;

        assert_eq!(rope.to_string(), "hello");

        Ok(())
    }

    #[test]
    fn test_apply_insert() -> Result<(), SyncLibError> {
        let mut rope = Rope::from_str("hello");
        let operation = Operation::create_insert(5, " my friend".to_string()).unwrap();

        operation.apply(&mut rope)?;

        assert_eq!(rope.to_string(), "hello my friend");

        Ok(())
    }
}
