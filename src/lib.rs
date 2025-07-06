//! # Reconcile
//!
//! [`diff3`](https://www.gnu.org/software/diffutils/manual/html_node/Invoking-diff3.html) (or `git merge`)
//! but with automatic conflict resolution.
//!
//! Reconcile is a Rust and JavaScript (through WebAssembly) library for merging
//! text without user intervention. It automatically resolves conflicts that
//! would typically require user action in traditional 3-way merge tools.
//!
//! Try out the [interactive demo](https://schmelczer.dev/reconcile)!
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
//! can't get jumbled up at the end of reconciling.
//!
//! ### Built-in tokenizers
//!
//! ```
//! use reconcile::{reconcile, BuiltinTokenizer};
//!
//! let parent = "The quick brown fox\n";
//! let left = "The very quick brown fox\n";
//! let right = "The quick red fox\n";
//!
//! let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Line);
//! assert_eq!(result.apply().text(), "The quick red foxThe very quick brown fox\n");
//! ```
//!
//! ### Custom tokenization
//!
//! If something custom is needed, for instance, to better support structured
//! text such as Markdown or HTML, a custom tokenizer can be implemented:
//!
//! ```
//! use reconcile::{reconcile, Token, BuiltinTokenizer};
//!
//! // Example with custom tokenizer - split by sentences
//! let sentence_tokenizer = |text: &str| {
//!     text.split(". ")
//!         .map(|sentence| Token::new(
//!             sentence.to_string(),
//!             sentence.to_string(),
//!             false, // don't allow joining token with the preceding one
//!             false, // don't allow joining token with the following one
//!         ))
//!         .collect::<Vec<_>>()
//! };
//!
//! let parent = "Hello world. This is a test.";
//! let left = "Hello beautiful world. This is a test.";
//! let right = "Hello world. This is a great test.";
//!
//! // Using built-in tokenizer is usually sufficient
//! let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Word);
//! assert_eq!(result.apply().text(), "Hello beautiful world. This is a great test.");
//! ```
//! > By setting the joinability to `false`, longer runs of inserts will be
//! > interleaved like LRLRLR instead of LLLRRR.
//!
//! ## Cursors and selection ranges
//!
//! The library supports updating cursor and selection ranges during the merging
//! for interactive workflows:
//!
//! ```
//! use reconcile::{reconcile, BuiltinTokenizer, TextWithCursors, CursorPosition};
//!
//! let parent = "Hello world";
//! let left = TextWithCursors::new(
//!     "Hello beautiful world".to_string(),
//!     vec![CursorPosition { id: 1, char_index: 6 }] // After "Hello "
//! );
//! let right = TextWithCursors::new(
//!     "Hi world".to_string(),
//!     vec![CursorPosition { id: 2, char_index: 0 }] // At beginning
//! );
//!
//! let result = reconcile(parent, &left, &right, &*BuiltinTokenizer::Word);
//! let merged = result.apply();
//!
//! assert_eq!(merged.text(), "Hi beautiful world");
//! // Cursors are automatically repositioned
//! assert_eq!(merged.cursors().len(), 2);
//! ```
//!
//! ## The algorithm
//!
//! For a discussion of the algorithm and architecture, see the
//! [README](README.md#algorithm) page.

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
