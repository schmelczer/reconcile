#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::History;

/// Wrapper type to expose `TextWithCursors` to JS.
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct JsTextWithCursors {
    text: String,
    cursors: Vec<JsCursorPosition>,
}

#[wasm_bindgen]
impl JsTextWithCursors {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(text: String, cursors: Vec<JsCursorPosition>) -> Self { Self { text, cursors } }

    #[must_use]
    pub fn text(&self) -> String { self.text.clone() }

    #[must_use]
    pub fn cursors(&self) -> Vec<JsCursorPosition> { self.cursors.clone() }
}

impl From<JsTextWithCursors> for crate::TextWithCursors<'_> {
    fn from(owned: JsTextWithCursors) -> Self {
        crate::TextWithCursors::new_owned(
            owned.text.to_string(),
            owned
                .cursors
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
        )
    }
}

impl From<crate::TextWithCursors<'_>> for JsTextWithCursors {
    fn from(text_with_cursors: crate::TextWithCursors<'_>) -> Self {
        JsTextWithCursors {
            text: text_with_cursors.text.into_owned(),
            cursors: text_with_cursors
                .cursors
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
        }
    }
}

/// Wrapper type to expose `CursorPosition` to JS.
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct JsCursorPosition {
    id: usize,
    char_index: usize,
}

#[wasm_bindgen]
impl JsCursorPosition {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(id: usize, char_index: usize) -> Self { Self { id, char_index } }

    #[must_use]
    pub fn id(&self) -> usize { self.id }

    #[wasm_bindgen(js_name = characterPosition)]
    #[must_use]
    pub fn char_index(&self) -> usize { self.char_index }
}

impl From<JsCursorPosition> for crate::CursorPosition {
    fn from(owned: JsCursorPosition) -> Self {
        crate::CursorPosition {
            id: owned.id,
            char_index: owned.char_index,
        }
    }
}

impl From<crate::CursorPosition> for JsCursorPosition {
    fn from(cursor: crate::CursorPosition) -> Self {
        JsCursorPosition {
            id: cursor.id,
            char_index: cursor.char_index,
        }
    }
}
