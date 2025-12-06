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
}
