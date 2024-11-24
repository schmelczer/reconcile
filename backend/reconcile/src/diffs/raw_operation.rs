use crate::tokenizer::token::Token;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq)]
pub enum RawOperation<T>
where
    T: PartialEq + Hash + Clone,
{
    Insert(Vec<Token<T>>),
    Delete(Vec<Token<T>>),
    Equal(Vec<Token<T>>),
}

impl<T> RawOperation<T>
where
    T: PartialEq + Hash + Clone,
{
    pub fn tokens(&self) -> &Vec<Token<T>> {
        match self {
            RawOperation::Insert(tokens) => tokens,
            RawOperation::Delete(tokens) => tokens,
            RawOperation::Equal(tokens) => tokens,
        }
    }

    pub fn original_text_length(&self) -> usize {
        self.tokens().iter().map(|t| t.get_original_length()).sum()
    }

    pub fn get_original_text(self) -> String {
        self.tokens().iter().map(|t| t.original()).collect()
    }

    /// Extends the operation with another operation if returning the new operation.
    /// Only operations of the same type can be used to extend. If the operations are of different
    /// types, returns None.
    pub fn extend(self, other: RawOperation<T>) -> Option<RawOperation<T>> {
        match (self, other) {
            (RawOperation::Insert(tokens1), RawOperation::Insert(tokens2)) => Some(
                RawOperation::Insert(tokens1.into_iter().chain(tokens2).collect()),
            ),
            (RawOperation::Delete(tokens1), RawOperation::Delete(tokens2)) => Some(
                RawOperation::Delete(tokens1.into_iter().chain(tokens2).collect()),
            ),
            (RawOperation::Equal(tokens1), RawOperation::Equal(tokens2)) => Some(
                RawOperation::Equal(tokens1.into_iter().chain(tokens2).collect()),
            ),
            _ => None,
        }
    }
}
