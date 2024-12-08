use base64::DecodeError;
use thiserror::Error;
use wasm_bindgen::JsValue;

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

impl From<SyncLibError> for JsValue {
    fn from(val: SyncLibError) -> Self { JsValue::from_str(&val.to_string()) }
}
