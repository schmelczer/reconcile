//! # Reconcile: conflict-free 3-way text merging
//!
//! A library for merging conflicting text edits without manual intervention.
//! Unlike traditional 3-way merge tools that produce conflict markers,
//! reconcile-text automatically resolves conflicts by applying both sets of
//! changes (while updating cursor positions) using an algorithm inspired by
//! Operational Transformation.
//!
//! ✨ **[Try the interactive demo](https://schmelczer.dev/reconcile)** to see it in action.
//!
//! ## Simple example
//!
//! ```
//! use reconcile_text::{reconcile, BuiltinTokenizer};
//!
//! // Start with original text
//! let parent = "Merging text is hard!";
//! // Two people edit simultaneously
//! let left = "Merging text is easy!";                       // Changed "hard" to "easy"
//! let right = "With reconcile, merging documents is hard!"; // Added prefix and changed word
//!
//! // Reconcile combines both changes intelligently
//! let result = reconcile(parent, &left.into(), &right.into(), &*BuiltinTokenizer::Word);
//! assert_eq!(result.apply().text(), "With reconcile, merging documents is easy!");
//! ```
//!
//! ## Tokenisation strategies
//!
//! Merging happens at the token level, and the choice of tokeniser
//! significantly affects merge quality and behaviour.
//!
//! ### Built-in tokenisers
//!
//! - **`BuiltinTokenizer::Word`** (recommended): Splits on word boundaries,
//!   preserving word integrity
//! - **`BuiltinTokenizer::Character`**: Character-level merging for
//!   fine-grained control
//! - **`BuiltinTokenizer::Line`**: Line-based merging, similar to traditional
//!   diff tools
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
//! assert_eq!(result.apply().text(), "The quick red foxThe very quick brown fox\njumps over the lazy dog");
//! ```
//!
//! ### Custom tokenisation
//!
//! For specialised use cases, such as structured languages, custom
//! tokenisation logic can be implemented by providing a function with the
//! signature `Fn(&str) -> Vec<Token<String>>`::
//!
//! ```
//! use reconcile_text::{reconcile, Token, BuiltinTokenizer};
//!
//! // Example: sentence-based tokeniser function
//! let sentence_tokeniser = |text: &str| {
//!     text.split_inclusive(". ")
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
//! let result = reconcile(parent, &left.into(), &right.into(), &sentence_tokeniser);
//! assert_eq!(result.apply().text(), "Hello beautiful world. This is a great test.");
//! ```
//!
//! > **Note**: Setting token joinability to `false` causes insertions to
//! > interleave (LRLRLR) rather than group together (LLLRRR), which often
//! > produces more natural-looking merged text.
//!
//! ## Cursor tracking
//!
//! Automatically repositions cursors and selection ranges during merging,
//! which is essential for collaborative editors:
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
//! // Cursor 2 stays at position 0 (at the beginning)
//! ```
//! > The `cursors` list is sorted by character position (not IDs).
//!
//! ## Change provenance
//!
//! Track which changes came from where:
//!
//! ```rust
//! use reconcile_text::{History, SpanWithHistory, BuiltinTokenizer, reconcile};
//!
//! let parent = "Merging text is hard!";
//! let left = "Merging text is easy!"; // Changed "hard" to "easy"
//! let right = "With reconcile, merging documents is hard!"; // Added prefix and changed word
//!
//! let result = reconcile(
//!     parent,
//!     &left.into(),
//!     &right.into(),
//!     &*BuiltinTokenizer::Word,
//! );
//!
//! assert_eq!(
//!     result.apply_with_history(),
//!     vec![
//!         SpanWithHistory::new("Merging text".to_string(), History::RemovedFromRight),
//!         SpanWithHistory::new(
//!             "With reconcile, merging documents".to_string(),
//!             History::AddedFromRight
//!         ),
//!         SpanWithHistory::new(" ".to_string(), History::Unchanged),
//!         SpanWithHistory::new("is".to_string(), History::Unchanged),
//!         SpanWithHistory::new(" hard!".to_string(), History::RemovedFromLeft),
//!         SpanWithHistory::new(" easy!".to_string(), History::AddedFromLeft),
//!     ]
//! );
//! ```
//! ## Efficiently serialize changes
//!
//! The edits can be serialized into a compact representation without the full
//! original text, making the size only depends on the changes made.
//!
//! ```rust
//! use reconcile_text::{EditedText, BuiltinTokenizer};
//! use serde_yaml;
//! use pretty_assertions::assert_eq;
//!
//!
//! let original = "Merging text is hard!";
//! let changes = "Merging text is easy with reconcile!";
//!
//! let result = EditedText::from_strings(
//!     original,
//!     &changes.into()
//! );
//!
//! let serialized = serde_yaml::to_string(&result.to_change_set()).unwrap();
//! assert_eq!(
//!     serialized,
//!     concat!(
//!         "operations:\n",
//!         "- 15\n",
//!         "- -6\n",
//!         "- ' easy with reconcile!'\n",
//!         "cursors: []\n"
//!     )
//! );
//!
//! let deserialized = serde_yaml::from_str(&serialized).unwrap();
//! let reconstructed = EditedText::from_change_set(
//!     original,
//!     deserialized,
//!     &*BuiltinTokenizer::Word
//! );
//! assert_eq!(
//!     reconstructed.apply().text(),
//!     "Merging text is easy with reconcile!"
//! );
//! ```
//!
//! ## Error handling
//!
//! The library is designed to be robust and will always produce a result, even
//! in edge cases. However, be aware that extremely large diffs may have
//! performance implications.
//!
//! ## Algorithm overview
//!
//! For detailed algorithm explanation, see the
//! [README](https://github.com/schmelczer/reconcile/blob/main/README.md#how-it-works).

mod operation_transformation;
mod raw_operation;
mod tokenizer;
mod types;
mod utils;

pub use operation_transformation::{ChangeSet, EditedText, reconcile};
pub use tokenizer::{BuiltinTokenizer, Tokenizer, token::Token};
pub use types::{
    cursor_position::CursorPosition, history::History, side::Side,
    span_with_history::SpanWithHistory, text_with_cursors::TextWithCursors,
};

#[cfg(feature = "wasm")]
pub mod wasm;
