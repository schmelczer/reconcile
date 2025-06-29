#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::types::history::History;

/// Wrapper type to expose `(History, String)` to JS.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct TextWithHistory {
    history: History,
    text: String,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl TextWithHistory {
    pub fn new(history: History, text: String) -> Self { TextWithHistory { history, text } }

    #[must_use]
    pub fn history(&self) -> History { self.history }

    #[must_use]
    pub fn text(&self) -> String { self.text.clone() }
}
