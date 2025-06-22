#![feature(stmt_expr_attributes)]

mod diffs;
mod operation_transformation;
mod tokenizer;
mod utils;

pub use operation_transformation::{
    CursorPosition, EditedText, TextWithCursors, reconcile, reconcile_with_cursors,
    reconcile_with_history, reconcile_with_tokenizer,
};
pub use tokenizer::{Tokenizer, token::Token, word_tokenizer::word_tokenizer};
pub use utils::{history::History, side::Side};

#[cfg(feature = "wasm")]
pub mod wasm;
