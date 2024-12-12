use std::str::Utf8Error;

use base64::DecodeError;
use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Error, Debug)]
pub enum SyncLibError {
    #[error("Base64 decoding error because of {}", .reason)]
    Base64DecodingError { reason: String },

    #[error("Bytes cannot be decoded as UTF-8 string because of {}", .reason)]
    StringDecodingError { reason: String },
}

impl From<DecodeError> for SyncLibError {
    fn from(e: DecodeError) -> Self {
        SyncLibError::Base64DecodingError {
            reason: e.to_string(),
        }
    }
}

impl From<Utf8Error> for SyncLibError {
    fn from(e: Utf8Error) -> Self {
        SyncLibError::StringDecodingError {
            reason: e.to_string(),
        }
    }
}

impl From<std::string::FromUtf8Error> for SyncLibError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        SyncLibError::Base64DecodingError {
            reason: e.to_string(),
        }
    }
}

impl From<SyncLibError> for JsValue {
    fn from(val: SyncLibError) -> Self { JsValue::from_str(&val.to_string()) }
}
