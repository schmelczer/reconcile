use core::str;

use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
use errors::SyncLibError;
use wasm_bindgen::prelude::*;

pub mod errors;

#[wasm_bindgen(js_name = bytesToBase64)]
pub fn bytes_to_base64(input: &[u8]) -> String { STANDARD_NO_PAD.encode(input) }

#[wasm_bindgen(js_name = stringToBase64)]
pub fn string_to_base64(input: &str) -> String { bytes_to_base64(input.as_bytes()) }

#[wasm_bindgen(js_name = base64ToBytes)]
pub fn base64_to_bytes(input: &str) -> Result<Vec<u8>, SyncLibError> {
    STANDARD_NO_PAD.decode(input).map_err(SyncLibError::from)
}

#[wasm_bindgen(js_name = base64ToString)]
pub fn base64_to_string(input: &str) -> Result<String, SyncLibError> {
    let bytes = base64_to_bytes(input)?;
    String::from_utf8(bytes).map_err(SyncLibError::from)
}

#[wasm_bindgen]
pub fn merge(parent: &[u8], left: &[u8], right: &[u8]) -> Result<Vec<u8>, SyncLibError> {
    Ok(if is_binary(right) {
        right.to_vec()
    } else {
        reconcile::reconcile(
            str::from_utf8(parent).map_err(SyncLibError::from)?,
            str::from_utf8(left).map_err(SyncLibError::from)?,
            str::from_utf8(right).map_err(SyncLibError::from)?,
        )
        .into_bytes()
    })
}

#[wasm_bindgen(js_name = mergeText)]
pub fn merge_text(parent: &str, left: &str, right: &str) -> String {
    reconcile::reconcile(parent, left, right)
}

#[wasm_bindgen(js_name = isBinary)]
pub fn is_binary(data: &[u8]) -> bool { std::str::from_utf8(data).is_ok() }

#[cfg(feature = "console_error_panic_hook")]
#[wasm_bindgen(js_name = setPanicHook)]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    console_error_panic_hook::set_once();
}
