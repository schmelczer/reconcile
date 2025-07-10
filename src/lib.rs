//! # Reconcile: 3-way text merging with automatic conflict resolution
//!
//! A library for merging conflicting text edits without manual intervention.
//! Unlike traditional 3-way merge tools that produce conflict markers, this library
//! automatically resolves conflicts by applying both sets of changes where possible.
//!
//! Based on a combination of Myers' diff algorithm and Operational Transformation
//! principles, it's designed for scenarios where you have a common parent text
//! and two modified versions that need to be intelligently combined.
//!
//! **[Try the interactive demo](https://schmelczer.dev/reconcile)** to see it in action.
//!
//! ## Basic usage
//!
//! ```
//! use reconcile_text::{reconcile, BuiltinTokenizer};
//!
//! // Start with original text
//! let parent = "Merging text is hard!";
//! // Two people edit simultaneously  
//! let left = "Merging text is easy!";                    // Changed "hard" to "easy"
//! let right = "With reconcile, merging documents is hard!"; // Added prefix and changed word
//!
//! // Reconcile combines both changes intelligently
//! let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Word);
//! assert_eq!(result.apply().text(), "With reconcile, merging documents is easy!");
//! ```
//!
//! ## Tokenisation strategies
//!
//! Merging operates at the token level, where you control the granularity.
//! The choice of tokeniser significantly affects merge quality and behaviour.
//!
//! ### Built-in tokenisers
//!
//! - **`BuiltinTokenizer::Word`** (recommended): Splits on word boundaries, preserving word integrity
//! - **`BuiltinTokenizer::Character`**: Character-level merging for fine-grained control
//! - **`BuiltinTokenizer::Line`**: Line-based merging, similar to traditional diff tools
//!
//! ```
//! use reconcile_text::{reconcile, BuiltinTokenizer};
//!
//! let parent = "The quick brown fox\njumps over the lazy dog";
//! let left = "The very quick brown fox\njumps over the lazy dog";   // Added "very"
//! let right = "The quick red fox\njumps over the lazy dog";         // Changed "brown" to "red"
//!
//! // Word-level tokenisation (recommended for most text)
//! let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Word);
//! assert_eq!(result.apply().text(), "The very quick red fox\njumps over the lazy dog");
//!
//! // Line-level tokenisation (similar to git merge)
//! let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Line);
//! // Line-level produces different results as it treats each line as atomic
//! ```
//!
//! ### Custom tokenisation
//!
//! For specialised use cases, implement custom tokenisation logic:
//!
//! ```
//! use reconcile_text::{reconcile, Token, BuiltinTokenizer};
//!
//! // Example: sentence-based tokeniser function
//! let sentence_tokeniser = |text: &str| {
//!     text.split(". ")
//!         .map(|sentence| Token::new(
//!             sentence.to_string(),
//!             sentence.to_string(),
//!             false, // don't allow joining with preceding token
//!             false, // don't allow joining with following token
//!         ))
//!         .collect::<Vec<_>>()
//! };
//!
//! let parent = "Hello world. This is a test.";
//! let left = "Hello beautiful world. This is a test.";  // Added "beautiful"
//! let right = "Hello world. This is a great test.";     // Changed "a" to "great"
//!
//! // For most cases, the built-in word tokeniser works well
//! let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Word);
//! assert_eq!(result.apply().text(), "Hello beautiful world. This is a great test.");
//! ```
//!
//! > **Note**: Setting token joinability to `false` causes insertions to interleave
//! > (LRLRLR) rather than group together (LLLRRR), which often produces more
//! > natural-looking merged text.
//!
//! ## Cursor tracking
//!
//! Automatically repositions cursors and selection ranges during merging,
//! essential for collaborative editors:
//!
//! ```
//! use reconcile_text::{reconcile, BuiltinTokenizer, TextWithCursors, CursorPosition};
//!
//! let parent = "Hello world";
//! let left = TextWithCursors::new(
//!     "Hello beautiful world".to_string(),
//!     vec![CursorPosition { id: 1, char_index: 6 }] // After "Hello "
//! );
//! let right = TextWithCursors::new(
//!     "Hi world".to_string(),
//!     vec![CursorPosition { id: 2, char_index: 0 }] // At the beginning
//! );
//!
//! let result = reconcile(parent, &left, &right, &*BuiltinTokenizer::Word);
//! let merged = result.apply();
//!
//! assert_eq!(merged.text(), "Hi beautiful world");
//! // Cursors are automatically repositioned in the merged text
//! assert_eq!(merged.cursors().len(), 2);
//! // Cursor 1 moves from position 6 to position 3 (after "Hi ")
//! // Cursor 2 stays at position 0 (beginning)
//! ```
//!
//! ## Error handling
//!
//! The library is designed to be robust and will always produce a result, even
//! in edge cases. However, be aware that:
//!
//! - Binary data is detected and handled gracefully
//! - Unicode text is fully supported
//! - Extremely large diffs may have performance implications
//!
//! ## Algorithm overview
//!
//! 1. **Diff computation**: Myers' algorithm calculates differences between parent↔left and parent↔right
//! 2. **Tokenisation**: Text is split into meaningful units (words, characters, etc.)
//! 3. **Diff optimisation**: Operations are reordered and consolidated for coherent changes
//! 4. **Operational Transformation**: Edits are combined using OT principles
//!
//! For detailed algorithm explanation, see the [README](README.md#how-it-works).

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
