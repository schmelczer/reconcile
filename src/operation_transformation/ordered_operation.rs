#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{operation_transformation::Operation, Token};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct OrderedOperation<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    pub order: usize,
    pub operation: Operation<T>,
}

impl<T> OrderedOperation<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    pub fn get_sort_key(&self) -> (usize, usize, usize, String) {
        (
            self.order,
            match &self.operation {
                Operation::Delete { .. } => 1,
                Operation::Insert { .. } => 2,
                Operation::Equal { .. } => 3,
            },
            self.operation.start_index(),
            // Make sure that the ordering is deterministic regardless of which text
            // is left or right.
            match &self.operation {
                Operation::Equal { index, .. } => index.to_string(),
                Operation::Insert { text, .. } => {
                    text.iter().map(Token::original).collect::<String>()
                }
                Operation::Delete {
                    deleted_character_count,
                    ..
                } => deleted_character_count.to_string(),
            },
        )
    }
}

impl<T> PartialOrd for OrderedOperation<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get_sort_key().partial_cmp(&other.get_sort_key())
    }
}
