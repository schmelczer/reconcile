use ropey::Rope;
use serde::{Deserialize, Serialize};
use similar::{Change, ChangeTag};
use std::cmp::Ordering;
use std::fmt::Display;

use crate::errors::SyncLibError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operation {
    Insert {
        index: i64,
        text: String,
    },

    Delete {
        index: i64,
        deleted_character_count: i64,
    },
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Insert { index, text } => {
                write!(f, "+\"{}\" at {}", text, index)
            }
            Operation::Delete {
                index,
                deleted_character_count,
            } => {
                write!(f, "-{} at {}", deleted_character_count, index)
            }
        }
    }
}

impl Operation {
    pub fn create(tag: ChangeTag, index: i64, text: &str) -> Result<Self, SyncLibError> {
        if index < 0 {
            return Err(SyncLibError::NegativeOperationIndexError(format!(
                "Index {} is negative",
                index
            )));
        }

        Ok(match tag {
            ChangeTag::Insert => Operation::Insert {
                index,
                text: text.to_string(),
            },
            ChangeTag::Delete => Operation::Delete {
                index,
                deleted_character_count: text.chars().count() as i64,
            },
            _ => {
                return Err(SyncLibError::OperationConversionError(format!(
                    "Cannot convert editing operation because {:?}",
                    tag
                )))
            }
        })
    }

    pub fn create_insert(index: i64, text: &str) -> Result<Self, SyncLibError> {
        Self::create(ChangeTag::Insert, index, text)
    }

    pub fn create_delete(index: i64, length: i64) -> Result<Self, SyncLibError> {
        if index < 0 {
            return Err(SyncLibError::NegativeOperationIndexError(format!(
                "Index {} is negative",
                index
            )));
        }

        if length < 0 {
            return Err(SyncLibError::NegativeOperationIndexError(format!(
                "Length {} is negative",
                length
            )));
        }

        Ok(Operation::Delete {
            index,
            deleted_character_count: length,
        })
    }

    pub fn apply<'a>(&self, rope_text: &'a mut Rope) -> Result<&'a mut Rope, SyncLibError> {
        let index: usize = self.start_index() as usize;
        match self {
            Operation::Insert { text, .. } => rope_text.try_insert(index, &text).map_err(|err| {
                SyncLibError::OperationApplicationError(format!("Failed to insert text: {}", err))
            }),
            Operation::Delete {
                deleted_character_count,
                ..
            } => rope_text
                .try_remove(index..index + *deleted_character_count as usize)
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
    pub fn start_index(&self) -> i64 {
        match self {
            Operation::Insert { index, .. } => *index,
            Operation::Delete { index, .. } => *index,
        }
    }

    /// Returns the index of the last character that the operation affects.
    pub fn end_index(&self) -> i64 {
        self.start_index() + self.len() - 1
    }

    /// Returns the number of affected characters.
    pub fn len(&self) -> i64 {
        match self {
            Operation::Insert { text, .. } => text.chars().count() as i64,
            Operation::Delete {
                deleted_character_count,
                ..
            } => *deleted_character_count,
        }
    }

    /// Returns the range of indices of characters that the operation affects, inclusive.
    pub fn range(&self) -> std::ops::RangeInclusive<i64> {
        self.start_index()..=self.end_index()
    }

    pub fn with_index(&self, index: i64) -> Self {
        match self {
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
        }
    }

    pub fn with_shifted_index(&self, offset: i64) -> Self {
        let new_index = 0.max(self.start_index() + offset);
        self.with_index(new_index)
    }
}

impl Ord for Operation {
    fn cmp(&self, other: &Self) -> Ordering {
        let result = self.start_index().cmp(&other.start_index());
        if result == Ordering::Equal {
            match (self, other) {
                (Operation::Insert { .. }, Operation::Delete { .. }) => Ordering::Greater,
                (Operation::Delete { .. }, Operation::Insert { .. }) => Ordering::Less,
                _ => Ordering::Equal,
            }
        } else {
            result
        }
    }
}

impl PartialOrd for Operation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation_errors() {
        insta::assert_debug_snapshot!(Operation::create(ChangeTag::Insert, -1, "hi"));
        insta::assert_debug_snapshot!(Operation::create(ChangeTag::Equal, 0, "hi"));
        insta::assert_debug_snapshot!(Operation::create_insert(-1, "hi"));
        insta::assert_debug_snapshot!(Operation::create_delete(0, -1));
        insta::assert_debug_snapshot!(Operation::create_delete(-1, -1));
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
        let operation = Operation::create(ChangeTag::Delete, 6, "world")?;

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
        let operation = Operation::create(ChangeTag::Insert, 5, " my friend")?;

        operation.apply(&mut rope)?;

        assert_eq!(rope.to_string(), "hello my friend");

        Ok(())
    }
}
