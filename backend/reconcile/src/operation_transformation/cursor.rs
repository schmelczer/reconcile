use std::borrow::Cow;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::merge_context::MergeContext;
use crate::operation_transformation::Operation;

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
    pub fn apply_merge_context<T>(&self, context: &MergeContext<T>) -> Self
    where
        T: PartialEq + Clone + std::fmt::Debug,
    {
        let char_index = match context.last_operation() {
            Some(Operation::Delete { index, .. }) => (*index) as i64,
            _ => self.char_index as i64 + context.shift,
        };

        CursorPosition {
            id: self.id,
            char_index: char_index.max(0) as usize,
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
