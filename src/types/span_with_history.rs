#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::types::history::History;

/// Wrapper type for `(String, History)` where History describes the origin of
/// `text`.
#[allow(clippy::unsafe_derive_deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct SpanWithHistory {
    text: String,
    history: History,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl SpanWithHistory {
    #[must_use]
    pub fn new(text: String, history: History) -> Self { SpanWithHistory { text, history } }

    #[must_use]
    pub fn history(&self) -> History { self.history }

    #[must_use]
    pub fn text(&self) -> String { self.text.clone() }
}
