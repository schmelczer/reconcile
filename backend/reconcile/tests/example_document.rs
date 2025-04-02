use std::{fs, path::Path};

use pretty_assertions::assert_eq;
use reconcile::{CursorPosition, TextWithCursors};
use serde::Deserialize;

/// `ExampleDocument` represents a test case for the reconciliation process.
/// It contains a parent string, left and right strings with cursor positions,
/// and the expected result after reconciliation.
///
/// '|' characters in the left, right, and expected strings are treated as
/// cursor positions and are converted into `CursorPosition` objects.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct ExampleDocument {
    parent: String,
    left: String,
    right: String,
    expected: String,
}

impl ExampleDocument {
    /// Creates a new `ExampleDocument` instance from a YAML file.
    ///
    /// # Panics
    ///
    /// If the file cannot be opened or parsed, the program will panic.
    #[must_use]
    pub fn from_yaml(path: &Path) -> Self {
        let file = fs::File::open(path).expect("Failed to open example file");
        serde_yaml::from_reader(file).expect("Failed to parse example file")
    }

    #[must_use]
    pub fn parent(&self) -> String { self.parent.clone() }

    #[must_use]
    pub fn left(&self) -> TextWithCursors<'static> {
        ExampleDocument::string_to_text_with_cursors(&self.left)
    }

    #[must_use]
    pub fn right(&self) -> TextWithCursors<'static> {
        ExampleDocument::string_to_text_with_cursors(&self.right)
    }

    /// Asserts that the result string matches the expected string,
    /// including cursor positions.
    ///
    /// # Panics
    ///
    /// If the result string does not match the expected string, the program
    /// will panic.
    pub fn assert_eq(&self, result: &TextWithCursors<'static>) {
        let result_str = ExampleDocument::text_with_cursors_to_string(result);
        assert_eq!(result_str, self.expected);
    }

    /// Asserts that the result string matches the expected string,
    /// ignoring cursor positions.
    ///
    /// # Panics
    ///
    /// If the result string does not match the expected string, the program
    /// will panic.
    pub fn assert_eq_without_cursors(&self, result: &str) {
        assert_eq!(
            result,
            ExampleDocument::string_to_text_with_cursors(&self.expected).text,
        );
    }

    fn text_with_cursors_to_string(text: &TextWithCursors<'_>) -> String {
        let mut result = text.text.clone().into_owned();
        for (i, cursor) in text.cursors.iter().enumerate() {
            result.insert(cursor.char_index + i, '|');
        }
        result
    }

    fn string_to_text_with_cursors(text: &str) -> TextWithCursors<'static> {
        let cursors = Self::parse_cursors(text);
        let text = text.replace('|', "");
        TextWithCursors::new_owned(text, cursors)
    }

    fn parse_cursors(text: &str) -> Vec<CursorPosition> {
        let mut cursors = Vec::new();
        for (i, c) in text.chars().enumerate() {
            if c == '|' {
                cursors.push(CursorPosition {
                    id: 0,
                    char_index: i - cursors.len(),
                });
            }
        }
        cursors
    }
}
