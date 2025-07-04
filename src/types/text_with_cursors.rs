#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::types::cursor_position::CursorPosition;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextWithCursors {
    text: String, // wasm-pack doesn't support generics so we can't use Cow here
    cursors: Vec<CursorPosition>,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl TextWithCursors {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    #[must_use]
    pub fn new(text: String, cursors: Vec<CursorPosition>) -> Self {
        let length = text.chars().count();
        for cursor in &cursors {
            debug_assert!(
                cursor.char_index <= length,
                // cursor.char_index == length means that the cursor is at the end
                "Cursor positions ({}) must be contained within the text (of length {length}) or \
                 just after the end",
                cursor.char_index
            );
        }

        Self { text, cursors }
    }

    #[must_use]
    pub fn text(&self) -> String { self.text.to_string() }

    #[must_use]
    pub fn cursors(&self) -> Vec<CursorPosition> { self.cursors.clone() }
}

impl<'a> From<&'a str> for TextWithCursors {
    fn from(text: &'a str) -> Self {
        Self {
            text: text.into(),
            cursors: Vec::new(),
        }
    }
}

impl From<&String> for TextWithCursors {
    fn from(text: &String) -> Self {
        Self {
            text: text.to_owned(),
            cursors: Vec::new(),
        }
    }
}

impl From<String> for TextWithCursors {
    fn from(text: String) -> Self {
        Self {
            text,
            cursors: Vec::new(),
        }
    }
}
