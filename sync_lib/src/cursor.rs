use wasm_bindgen::prelude::*;

/// Wrapper type to expose `TextWithCursors` to JS.
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct TextWithCursors {
    text: String,
    cursors: Vec<CursorPosition>,
}

#[wasm_bindgen]
impl TextWithCursors {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(text: String, cursors: Vec<CursorPosition>) -> Self { Self { text, cursors } }

    #[must_use]
    pub fn text(&self) -> String { self.text.clone() }

    #[must_use]
    pub fn cursors(&self) -> Vec<CursorPosition> { self.cursors.clone() }
}

impl From<TextWithCursors> for reconcile::TextWithCursors<'_> {
    fn from(owned: TextWithCursors) -> Self {
        reconcile::TextWithCursors::new_owned(
            owned.text.to_string(),
            owned
                .cursors
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
        )
    }
}

impl From<reconcile::TextWithCursors<'_>> for TextWithCursors {
    fn from(text_with_cursors: reconcile::TextWithCursors<'_>) -> Self {
        TextWithCursors {
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
pub struct CursorPosition {
    id: usize,
    char_index: usize,
}

#[wasm_bindgen]
impl CursorPosition {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(id: usize, char_index: usize) -> Self { Self { id, char_index } }

    #[must_use]
    pub fn id(&self) -> usize { self.id }

    #[wasm_bindgen(js_name = characterPosition)]
    #[must_use]
    pub fn char_index(&self) -> usize { self.char_index }
}

impl From<CursorPosition> for reconcile::CursorPosition {
    fn from(owned: CursorPosition) -> Self {
        reconcile::CursorPosition {
            id: owned.id,
            char_index: owned.char_index,
        }
    }
}

impl From<reconcile::CursorPosition> for CursorPosition {
    fn from(cursor: reconcile::CursorPosition) -> Self {
        CursorPosition {
            id: cursor.id,
            char_index: cursor.char_index,
        }
    }
}
