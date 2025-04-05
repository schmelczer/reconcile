#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::operation_transformation::Operation;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct OrderedOperation<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    pub order: usize,
    pub operation: Operation<T>,
}
