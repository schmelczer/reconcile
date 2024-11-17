use ropey::Rope;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::errors::SyncLibError;

/// Represents a change that can be applied to a text document.
/// Operation is tied to a ropey::Rope and is mainly expected to be
/// created by OperationSequence.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operation {
    Insert {
        index: usize,
        text: String,
    },

    Delete {
        index: usize,
        deleted_character_count: usize,
    },
}

impl Operation {
    /// Creates an insert operation with the given index and text.
    /// If the text is empty (meaning that the operation would be a no-op), returns None.
    pub fn create_insert(index: usize, text: &str) -> Result<Option<Self>, SyncLibError> {
        if text.is_empty() {
            return Ok(None);
        }

        Ok(Some(Operation::Insert {
            index,
            text: text.to_string(),
        }))
    }

    /// Creates a delete operation with the given index and number of to-be-deleted characters.
    /// If the operation would delete 0 (meaning that the operation would be a no-op), returns None.
    pub fn create_delete(
        index: usize,
        deleted_character_count: usize,
    ) -> Result<Option<Self>, SyncLibError> {
        if deleted_character_count == 0 {
            return Ok(None);
        }

        Ok(Some(Operation::Delete {
            index,
            deleted_character_count,
        }))
    }

    /// Tries to apply the operation to the given ropey::Rope text, returning the modified text.
    pub fn apply<'a>(&self, rope_text: &'a mut Rope) -> Result<&'a mut Rope, SyncLibError> {
        let index: usize = self.start_index();
        match self {
            Operation::Insert { text, .. } => rope_text.try_insert(index, text).map_err(|err| {
                SyncLibError::OperationApplicationError(format!("Failed to insert text: {}", err))
            }),
            Operation::Delete {
                deleted_character_count,
                ..
            } => rope_text
                .try_remove(index..index + { *deleted_character_count })
                .map_err(|err| {
                    SyncLibError::OperationApplicationError(format!(
                        "Failed to remove text: {}",
                        err
                    ))
                }),
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

    /// Returns the range of indices of characters that the operation affects, inclusive.
    pub fn range(&self) -> std::ops::RangeInclusive<usize> {
        self.start_index()..=self.end_index()
    }

    /// Clones the operation while updating the index.
    pub fn with_index(&self, index: usize) -> Result<Self, SyncLibError> {
        Ok(match self {
            Operation::Insert { text, .. } => Operation::Insert {
                index,
                text: text.clone(),
            },
            Operation::Delete {
                deleted_character_count,
                ..
            } => Operation::Delete {
                index,
                deleted_character_count: *deleted_character_count,
            },
        })
    }

    /// Clones the operation while shifting the index by the given offset.
    /// The offset can be negative but the resulting index must be non-negative.
    pub fn with_shifted_index(&self, offset: i64) -> Result<Self, SyncLibError> {
        let index = self.start_index() as i64 + offset;

        self.with_index(index.try_into().map_err(|_| {
            SyncLibError::NegativeOperationIndexError(format!(
                "Index {} is negative but operations must have a non-negative index",
                index
            ))
        })?)
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Insert { index, text } => {
                write!(f, "Insert '{}' from index {}", text, index)
            }
            Operation::Delete {
                index,
                deleted_character_count,
            } => {
                write!(
                    f,
                    "Delete {} characters index {}",
                    deleted_character_count, index
                )
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
        insta::assert_debug_snapshot!(Operation::create_insert(1, "hi")
            .unwrap()
            .unwrap()
            .with_shifted_index(-2));
    }

    #[test]
    fn test_apply_delete() -> Result<(), SyncLibError> {
        let mut rope = Rope::from_str("hello world");
        let operation = Operation::Delete {
            index: 5,
            deleted_character_count: 6,
        };

        operation.apply(&mut rope)?;

        assert_eq!(rope.to_string(), "hello");

        Ok(())
    }

    #[test]
    fn test_apply_delete_with_create() -> Result<(), SyncLibError> {
        let mut rope = Rope::from_str("hello world");
        let operation = Operation::create_delete(5, 6)?.unwrap();

        operation.apply(&mut rope)?;

        assert_eq!(rope.to_string(), "hello");

        Ok(())
    }

    #[test]
    fn test_apply_insert() -> Result<(), SyncLibError> {
        let mut rope = Rope::from_str("hello");
        let operation = Operation::Insert {
            index: 5,
            text: " my friend".to_string(),
        };

        operation.apply(&mut rope)?;

        assert_eq!(rope.to_string(), "hello my friend");

        Ok(())
    }

    #[test]
    fn test_apply_insert_with_create() -> Result<(), SyncLibError> {
        let mut rope = Rope::from_str("hello");
        let operation = Operation::create_insert(5, " my friend")?.unwrap();

        operation.apply(&mut rope)?;

        assert_eq!(rope.to_string(), "hello my friend");

        Ok(())
    }
}
