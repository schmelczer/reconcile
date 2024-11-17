use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncLibError {
    #[error("Failed to shift the operation's index {0}")]
    NegativeOperationIndexError(String),

    #[error("Failed to apply operation because {0}")]
    OperationApplicationError(String),
}
