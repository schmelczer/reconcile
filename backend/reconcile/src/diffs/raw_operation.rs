use crate::tokenizer::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum RawOperation {
    Insert(Vec<Token>),
    Delete(Vec<Token>),
    Equal(Vec<Token>),
}

impl RawOperation {
    pub fn tokens(&self) -> &Vec<Token> {
        match self {
            RawOperation::Insert(tokens) => tokens,
            RawOperation::Delete(tokens) => tokens,
            RawOperation::Equal(tokens) => tokens,
        }
    }

    pub fn original_text_length(&self) -> usize {
        self.tokens()
            .iter()
            .map(|t| t.original.chars().count())
            .sum()
    }

    pub fn get_original_text(self) -> String {
        self.tokens().iter().map(|t| t.original.clone()).collect()
    }

    /// Extends the operation with another operation if returning the new operation.
    /// Only operations of the same type can be used to extend. If the operations are of different
    /// types, returns None.
    pub fn extend(&self, other: &RawOperation) -> Option<RawOperation> {
        match (self, other) {
            (RawOperation::Insert(tokens1), RawOperation::Insert(tokens2)) => Some(
                RawOperation::Insert(tokens1.iter().chain(tokens2.iter()).cloned().collect()),
            ),
            (RawOperation::Delete(tokens1), RawOperation::Delete(tokens2)) => Some(
                RawOperation::Delete(tokens1.iter().chain(tokens2.iter()).cloned().collect()),
            ),
            (RawOperation::Equal(tokens1), RawOperation::Equal(tokens2)) => Some(
                RawOperation::Equal(tokens1.iter().chain(tokens2.iter()).cloned().collect()),
            ),
            _ => None,
        }
    }
}
