#![warn(clippy::all, clippy::restriction, clippy::pedantic, clippy::cargo)]

mod diffs;
mod operation_transformation;
mod tokenizer;
mod utils;

pub use operation_transformation::{reconcile, reconcile_with_tokenizer, EditedText};
pub use tokenizer::token::Token;
