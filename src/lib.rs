//! # Reconcile: conflict-free 3-way text merging
//!
//! Think [`diff3`](https://www.gnu.org/software/diffutils/manual/html_node/Invoking-diff3.html) or `git merge`,
//! but with intelligent conflict resolution.
//!
//! Reconcile is a Rust and JavaScript (via WebAssembly) library that merges
//! conflicting text edits without requiring manual intervention. Where
//! traditional 3-way merge tools would leave you with conflict markers to
//! resolve by hand, Reconcile automatically weaves changes together using
//! sophisticated algorithms inspired by Operational Transformation.
//!
//! ✨ **[Try the interactive demo](https://schmelczer.dev/reconcile)** to see it in action!
//!
//! ```
//! use reconcile::{reconcile, BuiltinTokenizer};
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
//! Merging happens at the token level, where you control the granularity.
//! By default, words serve as the atomic units for merging, ensuring words
//! remain intact during the reconciliation process.
//!
//! ### Built-in tokenisers
//!
//! ```
//! use reconcile::{reconcile, BuiltinTokenizer};
//!
//! let parent = "The quick brown fox\n";
//! let left = "The very quick brown fox\n";   // Added "very"
//! let right = "The quick red fox\n";         // Changed "brown" to "red"
//!
//! // Using line-based tokenisation
//! let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Line);
//! assert_eq!(result.apply().text(), "The quick red foxThe very quick brown fox\n");
//! ```
//!
//! ### Custom tokenisation
//!
//! For specialised use cases—such as structured text like Markdown or HTML—
//! you can implement custom tokenisation logic:
//!
//! ```
//! use reconcile::{reconcile, Token, BuiltinTokenizer};
//!
//! // Example: custom sentence-based tokeniser
//! let sentence_tokeniser = |text: &str| {
//!     text.split(". ")
//!         .map(|sentence| Token::new(
//!             sentence.to_string(),
//!             sentence.to_string(),
//!             false, // don't allow joining with the preceding token
//!             false, // don't allow joining with the following token
//!         ))
//!         .collect::<Vec<_>>()
//! };
//!
//! let parent = "Hello world. This is a test.";
//! let left = "Hello beautiful world. This is a test.";  // Added "beautiful"
//! let right = "Hello world. This is a great test.";     // Changed "a" to "great"
//!
//! // For most cases, the built-in word tokeniser works perfectly
//! let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Word);
//! assert_eq!(result.apply().text(), "Hello beautiful world. This is a great test.");
//! ```
//! > **Tip**: Setting joinability to `false` causes longer runs of insertions
//! > to interleave (LRLRLR) rather than group together (LLLRRR), which can
//! > produce more natural-looking merged text.
//!
//! ## Cursor tracking
//!
//! Perfect for collaborative editors—the library automatically repositions
//! cursors and selection ranges during merging:
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
//!     vec![CursorPosition { id: 2, char_index: 0 }] // At the beginning
//! );
//!
//! let result = reconcile(parent, &left, &right, &*BuiltinTokenizer::Word);
//! let merged = result.apply();
//!
//! assert_eq!(merged.text(), "Hi beautiful world");
//! // Cursors are automatically repositioned in the merged text
//! assert_eq!(merged.cursors().len(), 2);
//! ```
//!
//! ## How it works
//!
//! For a detailed explanation of the algorithm and architecture, see the
//! [README](README.md#how-it-works).

mod operation_transformation;
mod raw_operation;
mod tokenizer;
mod types;
mod utils;

pub use operation_transformation::{reconcile, EditedText};
pub use tokenizer::{token::Token, BuiltinTokenizer, Tokenizer};
pub use types::{
    cursor_position::CursorPosition, history::History, side::Side,
    span_with_history::SpanWithHistory, text_with_cursors::TextWithCursors,
};
pub use utils::is_binary::is_binary;

#[cfg(feature = "wasm")]
pub mod wasm;
