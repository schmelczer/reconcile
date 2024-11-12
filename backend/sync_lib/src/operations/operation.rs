use ropey::Rope;
use serde::{Deserialize, Serialize};
use similar::{Change, ChangeTag};
use std::cmp::Ordering;
use std::fmt::Display;

use crate::errors::SyncLibError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operation {
    Insert {
        index: u64,
        text: String,
    },

    Delete {
        index: u64,
        deleted_character_count: u64,
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

impl Default for Operation {
    fn default() -> Self {
        Operation::Insert {
            index: 0,
            text: "".to_string(),
        }
    }
}

impl Operation {
    pub fn new(tag: ChangeTag, index: u64, text: &str) -> Self {
        match tag {
            ChangeTag::Insert => Operation::Insert {
                index,
                text: text.to_string(),
            },
            ChangeTag::Delete => Operation::Delete {
                index,
                deleted_character_count: text.chars().count() as u64,
            },
            _ => panic!("Only insertion and deletions are supported"),
        }
    }
    pub fn apply<'a>(&self, rope_text: &'a mut Rope) -> Result<&'a mut Rope, SyncLibError> {
        let index: usize = self.index() as usize;
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

    pub fn index(&self) -> u64 {
        match self {
            Operation::Insert { index, .. } => *index,
            Operation::Delete { index, .. } => *index,
        }
    }

    pub fn with_index(&self, index: u64) -> Self {
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

    pub fn with_shifted_index(&self, offset: i64) -> Result<Self, SyncLibError> {
        let new_index = self.index().saturating_add_signed(offset);
        Ok(self.with_index(new_index))
    }
}

impl Ord for Operation {
    fn cmp(&self, other: &Self) -> Ordering {
        let result = self.index().cmp(&other.index());
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
}
