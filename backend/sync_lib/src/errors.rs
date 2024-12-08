use base64::DecodeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncLibError {
    #[error("Base64 decoding error: {}", .reason)]
    DecodingError { reason: String },
}

impl From<DecodeError> for SyncLibError {
    fn from(e: DecodeError) -> Self {
        SyncLibError::DecodingError {
            reason: e.to_string(),
        }
    }
}

impl From<std::string::FromUtf8Error> for SyncLibError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        SyncLibError::DecodingError {
            reason: e.to_string(),
        }
    }
}
