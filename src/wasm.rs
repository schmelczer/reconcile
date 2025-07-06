//! Expose the `reconcile` crate's functionality to WebAssembly.
use core::str;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

use crate::{BuiltinTokenizer, CursorPosition, SpanWithHistory, TextWithCursors};
cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;
    }
}

/// WASM wrapper around `crate::reconcile` for merging text.
#[wasm_bindgen(js_name = reconcile)]
#[must_use]
pub fn reconcile(
    parent: &str,
    left: &TextWithCursors,
    right: &TextWithCursors,
    tokenizer: BuiltinTokenizer,
) -> TextWithCursors {
    set_panic_hook();

    crate::reconcile(parent, left, right, &*tokenizer).apply()
}

/// WASM wrapper around `crate::reconcile` for merging text.
#[wasm_bindgen(js_name = reconcileWithHistory)]
#[must_use]
pub fn reconcile_with_history(
    parent: &str,
    left: &TextWithCursors,
    right: &TextWithCursors,
    tokenizer: BuiltinTokenizer,
) -> TextWithCursorsAndHistory {
    set_panic_hook();
    let reconciled = crate::reconcile(parent, left, right, &*tokenizer);
    let text_with_cursors = reconciled.apply();

    TextWithCursorsAndHistory {
        text_with_cursors,
        history: reconciled.apply_with_history(),
    }
}

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
#[wasm_bindgen(js_name = genericReconcile)]
#[must_use]
pub fn generic_reconcile(
    parent: &[u8],
    left: &[u8],
    right: &[u8],
    tokenizer: BuiltinTokenizer,
) -> Vec<u8> {
    set_panic_hook();

    if crate::is_binary(parent) || crate::is_binary(left) || crate::is_binary(right) {
        right.to_vec()
    } else {
        crate::reconcile(
            str::from_utf8(parent).expect("parent must be valid UTF-8 because it's not binary"),
            &str::from_utf8(left)
                .expect("left must be valid UTF-8 because it's not binary")
                .into(),
            &str::from_utf8(right)
                .expect("right must be valid UTF-8 because it's not binary")
                .into(),
            &*tokenizer,
        )
        .apply()
        .text()
        .into_bytes()
    }
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

/// WASM wrapper type for the return value of `reconcile_with_history`
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextWithCursorsAndHistory {
    text_with_cursors: TextWithCursors,
    history: Vec<SpanWithHistory>,
}

#[wasm_bindgen]
impl TextWithCursorsAndHistory {
    #[must_use]
    pub fn text(&self) -> String { self.text_with_cursors.text() }

    #[must_use]
    pub fn cursors(&self) -> Vec<CursorPosition> { self.text_with_cursors.cursors() }

    #[must_use]
    pub fn history(&self) -> Vec<SpanWithHistory> { self.history.clone() }
}
