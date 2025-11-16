//! Expose the `reconcile` crate's functionality to WebAssembly.
use core::str;

use wasm_bindgen::prelude::*;

use crate::{BuiltinTokenizer, CursorPosition, EditedText, SpanWithHistory, TextWithCursors};

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

/// WASM wrapper around getting a compact diff representation of two texts as a
/// list of numbers and strings.
#[wasm_bindgen(js_name = diff)]
#[must_use]
pub fn diff(parent: &str, changed: &TextWithCursors, tokenizer: BuiltinTokenizer) -> Vec<JsValue> {
    set_panic_hook();

    let edited_text = EditedText::from_strings_with_tokenizer(parent, changed, &*tokenizer);
    edited_text
        .to_diff()
        .into_iter()
        .map(std::convert::Into::into)
        .collect()
}

/// Inverse of `diff`, applies a compact diff representation to a parent text
#[wasm_bindgen(js_name = undiff)]
#[must_use]
pub fn undiff(parent: &str, diff: Vec<JsValue>, tokenizer: BuiltinTokenizer) -> String {
    set_panic_hook();

    EditedText::from_diff(
        parent,
        diff.into_iter()
            .map(|js_value| js_value.try_into())
            .collect::<Result<_, _>>()
            .expect("Invalid diff format"),
        &*tokenizer,
    )
    .apply()
    .text()
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

/// Returns the UTF8 parsed string if it's a text, or `None` if it's likely
/// binary.
#[must_use]
fn string_or_nothing(data: &[u8]) -> Option<String> {
    if data.contains(&0) {
        // Even though the NUL character is valid in UTF-8, it's highly suspicious in
        // human-readable text.
        return None;
    }

    std::str::from_utf8(data)
        .map(std::borrow::ToOwned::to_owned)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_or_nothing() {
        assert_eq!(string_or_nothing(&[0, 159, 146, 150]), None);
        assert_eq!(string_or_nothing(&[0, 12]), None);
        assert_eq!(string_or_nothing(b"hello"), Some("hello".into()));
    }
}
