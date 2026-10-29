use pretty_assertions::assert_eq;
use reconcile_text::{CursorPosition, EditedText, TextWithCursors};
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
    #[must_use]
    pub fn parent(&self) -> String {
        self.parent.clone()
    }

    #[must_use]
    pub fn left(&self) -> TextWithCursors {
        ExampleDocument::string_to_text_with_cursors(&self.left)
    }

    #[must_use]
    pub fn right(&self) -> TextWithCursors {
        ExampleDocument::string_to_text_with_cursors(&self.right)
    }

    /// Asserts that the result string matches the expected string,
    /// including cursor positions.
    ///
    /// # Panics
    ///
    /// If the result string does not match the expected string, the program
    /// will panic.
    pub fn assert_eq(&self, result: &EditedText<'_, String>) {
        assert_eq!(
            Self::string_to_text_with_cursors(&self.expected),
            result.apply()
        );
    }

    /// Asserts that the result string matches the expected string,
    /// ignoring cursor positions.
    ///
    /// # Panics
    ///
    /// If the result string does not match the expected string, the program
    /// will panic.
    pub fn assert_eq_without_cursors(&self, result: &str) {
        let expected = ExampleDocument::string_to_text_with_cursors(&self.expected).text();
        assert_eq!(
            expected, result,
            "Left (expected) isn't equal to right (actual), Actual: ```\n{result}```",
        );
    }

    fn string_to_text_with_cursors(text: &str) -> TextWithCursors {
        let cursors = Self::parse_cursors(text);
        let text = text.replace('|', "");
        TextWithCursors::new(text, cursors)
    }

    fn parse_cursors(text: &str) -> Vec<CursorPosition> {
        let mut cursors = Vec::new();
        for (i, c) in text.chars().enumerate() {
            if c == '|' {
                cursors.push(CursorPosition::new(0, i - cursors.len()));
            }
        }
        cursors
    }
}
