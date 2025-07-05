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
//! can't get jumbled up at the end of reconciling.
//!
//! ### Word-level tokenization (default)
//!
//! ```
//! use reconcile::{reconcile, BuiltinTokenizer};
//!
//! let parent = "The quick brown fox";
//! let left = "The very quick brown fox";
//! let right = "The quick red fox";
//!
//! let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Word);
//! assert_eq!(result.apply().text(), "The very quick red fox");
//! ```
//!
//! ### Character-level tokenization
//!
//! If finer grained merging is required, we can make every UTF-8 character
//! become its own token:
//!
//! ```
//! use reconcile::{reconcile, BuiltinTokenizer};
//!
//! let parent = "Hello";
//! let left = "Helo";    // deleted 'l'
//! let right = "Hello!"; // added '!'
//!
//! let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Character);
//! assert_eq!(result.apply().text(), "Helo!");
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
//!         .map(|sentence| Token::new(sentence.to_string(), sentence.to_string(), true, true))
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
//! The algorithm starts similarly to `diff3`. Its inputs are a **parent**
//! document and two conflicting versions: `left` and `right` which have
//! been created from the parent through any series of concurrent edits.
//!
//! When calling `reconcile(parent, left, right)`:
//!
//! 1. **Diff calculation**: 2-way diffs of (parent & left) and (parent & right)
//!    are computed using Myers' algorithm
//! 2. **Tokenization**: The text is split into tokens at the configured
//!    granularity
//! 3. **Operation transformation**: The resulting edits are weaved together
//!    using operational transformation principles, ensuring no changes are lost
//! 4. **Conflict resolution**: Unlike traditional merge tools, conflicts are
//!    automatically resolved without producing conflict markers
//!
//! The key insight is that both insertions and deletions are preserved:
//! - If either side inserted text, it appears in the result
//! - If either side deleted text, the deletion is applied
//! - Insertions into deleted regions are still preserved
//!
//! This approach works well for human-readable text where some "fuzziness" in
//! conflict resolution is acceptable, unlike source code where precision is
//! critical.

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
