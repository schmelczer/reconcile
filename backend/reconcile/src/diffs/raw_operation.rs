use crate::tokenizer::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum RawOperation<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    Insert(Vec<Token<T>>),
    Delete(Vec<Token<T>>),
    Equal(Vec<Token<T>>),
}

impl<T> RawOperation<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
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
        first_token.is_none_or(super::super::tokenizer::token::Token::get_is_left_joinable)
    }

    pub fn is_right_joinable(&self) -> bool {
        let last_token = self.tokens().last();
        last_token.is_none_or(super::super::tokenizer::token::Token::get_is_right_joinable)
    }

    /// Extends the operation with another operation. Only operations of the
    /// same type as self can be used to extend self, otherwise the function
    /// will panic.
    pub fn extend(self, other: RawOperation<T>) -> RawOperation<T> {
        debug_assert!(
            std::mem::discriminant(&self) == std::mem::discriminant(&other),
            "Cannot extend operations of different types. This should have been handled before \
             calling this function."
        );

        match (self, other) {
            (RawOperation::Insert(tokens1), RawOperation::Insert(tokens2)) => {
                RawOperation::Insert(tokens1.into_iter().chain(tokens2).collect())
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
