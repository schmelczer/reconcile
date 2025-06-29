use std::fmt::Debug;

use crate::{tokenizer::token::Token, utils::myers_diff::myers_diff};

/// Text editing operation containing the to-be-changed `Tokens`-s.
///
/// RawOperations can be joined together when the underlying tokens
/// allow for joining subseqeunt operations.
#[derive(Debug, Clone, PartialEq)]
pub enum RawOperation<T>
where
    T: PartialEq + Clone + Debug,
{
    Insert(Vec<Token<T>>),
    Delete(Vec<Token<T>>),
    Equal(Vec<Token<T>>),
}

impl<T> RawOperation<T>
where
    T: PartialEq + Clone + Debug,
{
    pub fn vec_from(left: &[Token<T>], right: &[Token<T>]) -> Vec<Self> { myers_diff(left, right) }

    pub fn tokens(&self) -> &Vec<Token<T>> {
        match self {
            RawOperation::Insert(tokens)
            | RawOperation::Delete(tokens)
            | RawOperation::Equal(tokens) => tokens,
        }
    }

    pub fn original_text_length(&self) -> usize {
        self.tokens().iter().map(Token::get_original_length).sum()
    }

    pub fn get_original_text(self) -> String { self.tokens().iter().map(Token::original).collect() }

    pub fn is_left_joinable(&self) -> bool {
        let first_token = self.tokens().first();
        first_token.is_none_or(|token| token.is_left_joinable)
    }

    pub fn is_right_joinable(&self) -> bool {
        let last_token = self.tokens().last();
        last_token.is_none_or(|token| token.is_right_joinable)
    }

    /// Extends the operation with another operation. Only operations of the
    /// same type as self can be used to extend self, otherwise the function
    /// will panic.
    pub fn join(self, other: RawOperation<T>) -> RawOperation<T> {
        debug_assert!(
            std::mem::discriminant(&self) == std::mem::discriminant(&other),
            "Cannot extend operations of different types. This should have been handled before \
             calling this function."
        );

        match (self, other) {
            (RawOperation::Insert(self_tokens), RawOperation::Insert(other_tokens)) => {
                RawOperation::Insert(self_tokens.into_iter().chain(other_tokens).collect())
            }
            (RawOperation::Delete(tokens1), RawOperation::Delete(tokens2)) => {
                RawOperation::Delete(tokens1.into_iter().chain(tokens2).collect())
            }
            (RawOperation::Equal(tokens1), RawOperation::Equal(tokens2)) => {
                RawOperation::Equal(tokens1.into_iter().chain(tokens2).collect())
            }
            _ => unreachable!("Only operations of the same type can be extended"),
        }
    }
}
