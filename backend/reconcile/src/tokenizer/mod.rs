use token::Token;

pub mod token;
pub mod word_tokenizer;

pub type Tokenizer<T> = dyn Fn(&str) -> Vec<Token<T>>;
