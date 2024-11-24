#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::operations::Operation;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderedOperation {
    pub order: usize,
    pub operation: Operation,
}
