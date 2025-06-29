#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::types::history::History;

/// Wrapper type for `(History, String)`
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct TextWithHistory {
    history: History,
    text: String,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl TextWithHistory {
    #[must_use]
    pub fn new(history: History, text: String) -> Self { TextWithHistory { history, text } }

    #[must_use]
    pub fn history(&self) -> History { self.history }

    #[must_use]
    pub fn text(&self) -> String { self.text.clone() }
}
