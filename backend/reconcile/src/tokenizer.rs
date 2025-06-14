use token::Token;

pub mod token;
pub mod word_tokenizer;

/// A trait for tokenizers that take a string and return a list of tokens.
pub type Tokenizer<T> = dyn Fn(&str) -> Vec<Token<T>>;
