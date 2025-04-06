use std::borrow::Cow;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// CursorPosition represents the position of an identifiable cursor in a text
// document based on its (UTF-8) character index.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CursorPosition {
    pub id: usize,
    pub char_index: usize,
}

impl CursorPosition {
    #[must_use]
    pub fn with_index(self, index: usize) -> Self {
        CursorPosition {
            id: self.id,
            char_index: index,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextWithCursors<'a> {
    pub text: Cow<'a, str>,
    pub cursors: Vec<CursorPosition>,
}

impl<'a> TextWithCursors<'a> {
    #[must_use]
    pub fn new(text: &'a str, cursors: Vec<CursorPosition>) -> Self {
        Self {
            text: text.into(),
            cursors,
        }
    }

    #[must_use]
    pub fn new_owned(text: String, cursors: Vec<CursorPosition>) -> Self {
        Self {
            text: text.into(),
            cursors,
        }
    }
}

impl<'a> From<&'a str> for TextWithCursors<'a> {
    fn from(text: &'a str) -> Self {
        Self {
            text: text.into(),
            cursors: Vec::new(),
        }
    }
}
