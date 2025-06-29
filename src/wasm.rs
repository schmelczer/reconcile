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

use crate::{
    TextWithCursors, TextWithHistory, reconcile, reconcile_with_cursors, reconcile_with_history,
};

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
        reconcile(
            str::from_utf8(parent).expect("parent must be valid UTF-8 because it's not binary"),
            str::from_utf8(left).expect("left must be valid UTF-8 because it's not binary"),
            str::from_utf8(right).expect("right must be valid UTF-8 because it's not binary"),
        )
        .into_bytes()
    }
}

/// WASM wrapper around `reconcile` for merging text.
#[wasm_bindgen(js_name = mergeText)]
#[must_use]
pub fn merge_text(parent: &str, left: &str, right: &str) -> String {
    set_panic_hook();

    reconcile(parent, left, right)
}

/// WASM wrapper around `reconcile` for merging text.
#[wasm_bindgen(js_name = mergeTextWithHistory)]
#[must_use]
pub fn merge_text_with_history(parent: &str, left: &str, right: &str) -> Vec<TextWithHistory> {
    set_panic_hook();

    reconcile_with_history(parent, left, right)
        .into_iter()
        .collect()
}

/// WASM wrapper around `reconcile::reconcile_with_cursors` for merging text.
#[wasm_bindgen(js_name = mergeTextWithCursors)]
#[must_use]
pub fn merge_text_with_cursors(
    parent: &str,
    left: &TextWithCursors,
    right: &TextWithCursors,
) -> TextWithCursors {
    set_panic_hook();

    reconcile_with_cursors(parent, left, right)
}

/// Heuristically determine if the given data is a binary or a text file's
/// content.
#[wasm_bindgen(js_name = isBinary)]
#[must_use]
pub fn is_binary(data: &[u8]) -> bool {
    set_panic_hook();
    crate::is_binary(data)
}

fn set_panic_hook() {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
