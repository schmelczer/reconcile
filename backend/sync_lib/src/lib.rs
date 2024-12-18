use core::str;

use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
use errors::SyncLibError;
use wasm_bindgen::prelude::*;

pub mod errors;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn bytes_to_base64(input: &[u8]) -> String { STANDARD_NO_PAD.encode(input) }

#[wasm_bindgen]
pub fn string_to_base64(input: &str) -> String { bytes_to_base64(input.as_bytes()) }

#[wasm_bindgen]
pub fn base64_to_bytes(input: &str) -> Result<Vec<u8>, SyncLibError> {
    STANDARD_NO_PAD.decode(input).map_err(SyncLibError::from)
}

#[wasm_bindgen]
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

#[wasm_bindgen]
pub fn is_binary(data: &[u8]) -> bool { data.iter().any(|&b| b == 0) }

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
