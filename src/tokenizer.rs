mod character_tokenizer;
mod word_tokenizer;

use std::ops::Deref;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use token::Token;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

pub mod token;

/// A trait for tokenizers that take a string and return a list of tokens.
pub type Tokenizer<T> = dyn Fn(&str) -> Vec<Token<T>>;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(feature = "wasm")]
pub enum BuiltinTokenizer {
    Character = "Character",
    Word = "Word",
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(not(feature = "wasm"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BuiltinTokenizer {
    Character,
    Word,
}

impl Deref for BuiltinTokenizer {
    type Target = Tokenizer<String>;

    fn deref(&self) -> &Self::Target {
        match self {
            BuiltinTokenizer::Character => &character_tokenizer::character_tokenizer,
            BuiltinTokenizer::Word => &word_tokenizer::word_tokenizer,
            #[cfg(feature = "wasm")]
            BuiltinTokenizer::__Invalid => panic!("Unexpected tokenizer type"),
        }
    }
}
