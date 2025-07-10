#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// `CursorPosition` represents the position of an identifiable cursor in a text
/// document based on its (UTF-8) character index.
#[allow(clippy::unsafe_derive_deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CursorPosition {
    pub id: usize,
    pub char_index: usize,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl CursorPosition {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    #[must_use]
    pub fn new(id: usize, char_index: usize) -> Self { Self { id, char_index } }

    #[must_use]
    pub fn with_index(&self, index: usize) -> Self {
        CursorPosition {
            id: self.id,
            char_index: index,
        }
    }

    #[must_use]
    pub fn id(&self) -> usize { self.id }

    #[cfg_attr(feature = "wasm", wasm_bindgen(js_name = characterIndex))]
    #[must_use]
    pub fn char_index(&self) -> usize { self.char_index }
}
