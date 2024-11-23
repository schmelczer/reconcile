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
}
