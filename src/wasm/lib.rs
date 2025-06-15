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

use wasm_bindgen::prelude::*;

use crate::wasm::cursor::JsTextWithCursors;

/// Merge two documents with a common parent. Relies on `reconcile::reconcile`
/// for texts and returns the right document as-is if either of the updated
/// documents is binary.
///
/// # Arguments
///
/// - `parent`: The common parent document.
/// - `left`: The left document updated by one user.
/// - `right`: The right document updated by another user.
///
/// # Returns
///
/// The merged document.
///
/// # Panics
///
/// If any of the input documents are not valid UTF-8 strings.
#[wasm_bindgen]
#[must_use]
pub fn merge(parent: &[u8], left: &[u8], right: &[u8]) -> Vec<u8> {
    set_panic_hook();

    if is_binary(parent) || is_binary(left) || is_binary(right) {
        right.to_vec()
    } else {
        crate::reconcile(
            str::from_utf8(parent).expect("parent must be valid UTF-8 because it's not binary"),
            str::from_utf8(left).expect("left must be valid UTF-8 because it's not binary"),
            str::from_utf8(right).expect("right must be valid UTF-8 because it's not binary"),
        )
        .into_bytes()
    }
}

/// WASM wrapper around `crate::reconcile` for merging text.
#[wasm_bindgen(js_name = mergeText)]
#[must_use]
pub fn merge_text(parent: &str, left: &str, right: &str) -> String {
    set_panic_hook();

    crate::reconcile(parent, left, right)
}

/// WASM wrapper around `reconcile::reconcile_with_cursors` for merging text.
#[wasm_bindgen(js_name = mergeTextWithCursors)]
#[must_use]
pub fn merge_text_with_cursors(
    parent: &str,
    left: JsTextWithCursors,
    right: JsTextWithCursors,
) -> JsTextWithCursors {
    set_panic_hook();

    crate::reconcile_with_cursors(parent, left.into(), right.into()).into()
}

/// Heuristically determine if the given data is a binary or a text file's
/// content.
#[wasm_bindgen(js_name = isBinary)]
#[must_use]
pub fn is_binary(data: &[u8]) -> bool {
    set_panic_hook();

    if data.contains(&0) {
        // Even though the NUL character is valid in UTF-8, it's highly suspicious in
        // human-readable text.
        return true;
    }

    std::str::from_utf8(data).is_err()
}

/// We don't want to support merging structured data like JSON, YAML, etc.
#[wasm_bindgen(js_name = isFileTypeMergable)]
#[must_use]
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
