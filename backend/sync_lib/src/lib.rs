use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD};
use errors::SyncLibError;

pub mod errors;

pub fn bytes_to_base64(input: &[u8]) -> String {
    STANDARD_NO_PAD.encode(input)
}

pub fn string_to_base64(input: &str) -> String {
    bytes_to_base64(input.as_bytes())
}

pub fn base64_to_bytes(input: &str) -> Result<Vec<u8>, SyncLibError> {
    STANDARD_NO_PAD.decode(input).map_err(SyncLibError::from)
}

pub fn base64_to_string(input: &str) -> Result<String, SyncLibError> {
    let bytes = base64_to_bytes(input)?;
    String::from_utf8(bytes).map_err(SyncLibError::from)
}

pub fn is_binary(data: &[u8]) -> bool {
    data.iter().any(|&b| b == 0)
}
