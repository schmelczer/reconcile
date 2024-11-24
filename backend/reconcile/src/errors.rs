use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncLibError {
    #[error("Failed to apply operation because {0}")]
    OperationApplicationError(String),
}
