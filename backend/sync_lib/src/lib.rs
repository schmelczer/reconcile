//! This crate provides utilities for easily communicating between backend &
//! frontend and ensuring the same logic for encoding and decoding binary data,
//! and 3-way-merging documents in Rust and JavaScript.
//!
//! The crate is designed to be used as a Rust library and as a
//! TypeScript/JavaScript package through WebAssembly (WASM).
//!
//! # Modules
//!
//! - `errors`: Contains error types used in this crate.
use core::str;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use errors::SyncLibError;
use wasm_bindgen::prelude::*;

pub mod errors;

/// Encode binary data for easy transport over HTTP. Inverse of
/// `base64_to_bytes`.
#[wasm_bindgen(js_name = bytesToBase64)]
pub fn bytes_to_base64(input: &[u8]) -> String {
    set_panic_hook();

    STANDARD.encode(input)
}

/// Inverse of `bytes_to_base64`.
#[wasm_bindgen(js_name = base64ToBytes)]
pub fn base64_to_bytes(input: &str) -> Result<Vec<u8>, SyncLibError> {
    set_panic_hook();

    STANDARD.decode(input).map_err(SyncLibError::from)
}

/// Merge two documents with a common parent. Relies on `reconcile::reconcile`
/// for texts and returns the right document as-is if either of the updated
/// documents is binary.
#[wasm_bindgen]
pub fn merge(parent: &[u8], left: &[u8], right: &[u8]) -> Vec<u8> {
    set_panic_hook();

    if is_binary(parent) || is_binary(left) || is_binary(right) {
        right.to_vec()
    } else {
        reconcile::reconcile(
            str::from_utf8(parent).expect("parent must be valid UTF-8 because it's not binary"),
            str::from_utf8(left).expect("left must be valid UTF-8 because it's not binary"),
            str::from_utf8(right).expect("right must be valid UTF-8 because it's not binary"),
        )
        .into_bytes()
    }
}

/// WASM wrapper around `reconcile::reconcile` for text merging.
#[wasm_bindgen(js_name = mergeText)]
pub fn merge_text(parent: &str, left: &str, right: &str) -> String {
    set_panic_hook();

    reconcile::reconcile(parent, left, right)
}

/// Heuristically determine if the given data is a binary or a text file's
/// content.
#[wasm_bindgen(js_name = isBinary)]
pub fn is_binary(data: &[u8]) -> bool {
    set_panic_hook();

    if data.iter().any(|&b| b == 0) {
        // Even though the NUL character is valid in UTF-8, it's highly suspicious in
        // human-readable text.
        return true;
    }

    std::str::from_utf8(data).is_err()
}

/// We don't want to support merging structured data like JSON, YAML, etc.
#[wasm_bindgen(js_name = isFileTypeMergable)]
pub fn is_file_type_mergable(path_or_file_name: &str) -> bool {
    set_panic_hook();

    let file_extension = path_or_file_name.split('.').next_back().unwrap_or_default();

    matches!(file_extension.to_lowercase().as_str(), "md" | "txt")
}

fn set_panic_hook() {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
