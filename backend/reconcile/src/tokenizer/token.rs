#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A token is a string that has been normalised in some way.
/// The normalised form is used for comparison, while the original form is used
/// for applying Operations.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Token<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    normalised: T,
    original: String,
}

impl From<&str> for Token<String> {
    fn from(s: &str) -> Self { Token::new(s.trim().to_owned(), s.to_owned()) }
}

impl<T> Token<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    pub fn new(normalised: T, original: String) -> Self {
        Token {
            normalised,
            original,
        }
    }

    pub fn original(&self) -> &str { &self.original }

    pub fn normalised(&self) -> &T { &self.normalised }

    pub fn get_original_length(&self) -> usize { self.original.chars().count() }
}

impl<T> PartialEq for Token<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    fn eq(&self, other: &Self) -> bool { self.normalised == other.normalised }
}
