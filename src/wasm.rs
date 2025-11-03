//! Expose the `reconcile` crate's functionality to WebAssembly.
use core::str;

use wasm_bindgen::prelude::*;

use crate::{
    BuiltinTokenizer, CursorPosition, SpanWithHistory, TextWithCursors,
    utils::string_or_nothing::string_or_nothing,
};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

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
#[wasm_bindgen(js_name = genericReconcile)]
#[must_use]
pub fn generic_reconcile(
    parent: &[u8],
    left: &[u8],
    right: &[u8],
    tokenizer: BuiltinTokenizer,
) -> Vec<u8> {
    set_panic_hook();

    if let (Some(parent), Some(left), Some(right)) = (
        string_or_nothing(parent),
        string_or_nothing(left),
        string_or_nothing(right),
    ) {
        crate::reconcile(&parent, &left.into(), &right.into(), &*tokenizer)
            .apply()
            .text()
            .into_bytes()
    } else {
        right.to_vec()
    }
}

/// WASM wrapper around getting a compact diff representation as a JSON string
///
/// # Panics
///
/// If serialization to JSON fails which should not happen
#[wasm_bindgen(js_name = getCompactDiff)]
#[must_use]
pub fn get_compact_diff(
    parent: &str,
    changed: &TextWithCursors,
    tokenizer: BuiltinTokenizer,
) -> String {
    set_panic_hook();
    let edited_text = crate::EditedText::from_strings_with_tokenizer(parent, changed, &*tokenizer);
    let change_set = edited_text.to_change_set();

    serde_json::to_string(&change_set).expect("Failed to serialize change set")
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
