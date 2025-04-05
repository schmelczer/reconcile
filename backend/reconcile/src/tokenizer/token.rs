#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A token is a string that has been normalised in some way.
/// The normalised form is used for comparison, while the original form is used
/// for applying `Operation`-s.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Token<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    /// The normalised form of the token used deriving the diff.
    pub normalised: T,

    /// The original string, that should be inserted or deleted in the document.
    original: String,

    /// Whether the token is joinable with the previous token.
    is_left_joinable: bool,

    /// Whether the token is joinable with the next token.
    is_right_joinable: bool,
}

impl From<&str> for Token<String> {
    fn from(text: &str) -> Self { Token::new(text.to_owned(), text.to_owned(), true, true) }
}

impl<T> Token<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    pub fn new(
        normalised: T,
        original: String,
        is_left_joinable: bool,
        is_right_joinable: bool,
    ) -> Self {
        Token {
            normalised,
            original,
            is_left_joinable,
            is_right_joinable,
        }
    }

    pub fn original(&self) -> &str { &self.original }

    pub fn normalised(&self) -> &T { &self.normalised }

    pub fn get_original_length(&self) -> usize { self.original.chars().count() }

    pub fn get_is_left_joinable(&self) -> bool { self.is_left_joinable }

    pub fn get_is_right_joinable(&self) -> bool { self.is_right_joinable }
}

impl<T> PartialEq for Token<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    fn eq(&self, other: &Self) -> bool { self.normalised == other.normalised }
}
