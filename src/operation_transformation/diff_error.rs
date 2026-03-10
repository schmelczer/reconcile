use thiserror::Error;

/// Error type for invalid diff operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum DiffError {
    /// The diff references a range that exceeds the original text length
    #[error(
        "Invalid diff: attempting to access {requested} characters starting at position \
         {position}, but original text only has {available} characters remaining"
    )]
    LengthExceedsOriginal {
        /// The position where the operation starts
        position: usize,
        /// The number of characters requested
        requested: usize,
        /// The number of characters available from the position
        available: usize,
    },

    /// A character count was too large to represent as i64
    #[error("Integer overflow: value {value} cannot be represented as i64")]
    IntegerOverflow {
        /// The value that caused the overflow
        value: usize,
    },
}
