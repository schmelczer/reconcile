//! # Reconcile
//!
//! A library for automatically merging two conflicting versions of a
//! document. `Reconcile` is essentially `git merge` but without any conflict
//! markers (or lost edits) in the output.
//!
//! ```
//! use reconcile::{reconcile, BuiltinTokenizer};
//!
//! let parent = "Merging text is hard!";
//! let left = "Merging text is easy!";
//! let right = "With reconcile, merging documents is hard!";
//!
//! let deconflicted = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Word);
//! assert_eq!(deconflicted.apply().text(), "With reconcile, merging documents is easy!");
//! ```
//! > You can also try out an interactive demo at [schmelczer.dev/reconcile](https://schmelczer.dev/reconcile).
//!
//! ## Tokenizing
//!
//! Merging is done on the token level, the granularity of which is
//! configurable. By default, words are the atoms for merging and thus words
//! can't get jumbled up at the end of reconciling. However, to maintain
//! gramatical correctness after merging, we could choose to treat individual
//! sentences as tokens:
//!
//! ```
//! ```
//!
//! > Beware, that if conflicting edits happen within a sentence (therefore each
//! > creating a new token), the sentences will appear duplicated.
//!
//! ```
//! ```
//!
//! If finer grained merging is required, we can make every UTF-8 character
//! become its own token:
//!
//!
//! If something custom is needed, for instance, to better support structured
//! text such as Markdown or HTML, a custom tokenizer can be implemented
//!
//!
//! ## Cursors and selection ranges
//!
//! Additionally, it supports updating cursor &
//! selection ranges during the merging too for interactive workflows.
//!
//!
//! ## The algorithm
//!
//! The algorithm starts similarly to `diff3`. Its inputs are a **Parent**
//! document `P` and two conflicting versions: `left` and `right` which have
//! been created from `P` through any series of concurrent edits. When calling
//! `reconcile(parent, left, right)`, first, the 2-way diff of (`parent` &
//! `left`) and (`parent` & `right`) are taken using Myers' algorithm.
//!
//! The
//!
//! Then, the
//! resulting edits are weaved together using the principles of operational
//! transformations ensuring that no change from either `left` or `right` is
//! lost: if either inserted some text, that string will end up in the result
//! and similarly for deletes.
//!
//! The
//!
//! The `reconcile` library
//!  

mod operation_transformation;
mod raw_operation;
mod tokenizer;
mod types;
mod utils;

pub use operation_transformation::{EditedText, reconcile};
pub use tokenizer::{BuiltinTokenizer, Tokenizer, token::Token};
pub use types::{
    cursor_position::CursorPosition, history::History, side::Side,
    span_with_history::SpanWithHistory, text_with_cursors::TextWithCursors,
};
pub use utils::is_binary::is_binary;

#[cfg(feature = "wasm")]
pub mod wasm;
