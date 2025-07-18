use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A token is a string that has been normalized in some way.
///
/// A token consists of the normalized form is used for comparison, and the
/// original form used for subsequently applying `Operation`-s to a text
/// document.
///
/// It's UTF-8 compatible.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Token<T>
where
    T: PartialEq + Clone + Debug,
{
    /// The normalized form of the token used deriving the diff.
    normalized: T,

    /// The original string, that should be inserted or deleted in the document.
    original: String,

    /// Whether the token is semantically joinable with the previous token.
    pub is_left_joinable: bool,

    /// Whether the token is semantically joinable with the next token.
    pub is_right_joinable: bool,
}

/// Trivial implementation of Token when the normalized form is the same as the
/// original string.
impl From<&str> for Token<String> {
    fn from(text: &str) -> Self { Token::new(text.to_owned(), text.to_owned(), true, true) }
}

impl<T> Token<T>
where
    T: PartialEq + Clone + Debug,
{
    pub fn new(
        normalized: T,
        original: String,
        is_left_joinable: bool,
        is_right_joinable: bool,
    ) -> Self {
        Token {
            normalized,
            original,
            is_left_joinable,
            is_right_joinable,
        }
    }

    pub fn original(&self) -> &str { &self.original }

    pub fn set_normalized(&mut self, normalized: T) { self.normalized = normalized; }

    pub fn normalized(&self) -> &T { &self.normalized }

    pub fn get_original_length(&self) -> usize { self.original.chars().count() }
}

impl<T> PartialEq for Token<T>
where
    T: PartialEq + Clone + Debug,
{
    fn eq(&self, other: &Self) -> bool { self.normalized == other.normalized }
}
