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
        assert_eq!(
            self.expected, result_str,
            "Left (expected) isn't equal to right (actual). Actual: ```\n{result_str}```",
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
        let expected = ExampleDocument::string_to_text_with_cursors(&self.expected).text;
        assert_eq!(
            expected, result,
            "Left (expected) isn't equal to right (actual), Actual: ```\n{result}```",
        );
    }

    fn text_with_cursors_to_string(text: &TextWithCursors<'_>) -> String {
        let mut result = text.text.clone().into_owned();
        for (i, cursor) in text.cursors.iter().enumerate() {
            assert!(
                cursor.char_index <= result.len(), // equals in case of insert at the end
                "Cursor index out of bounds: {} > {}",
                cursor.char_index,
                result.len()
            );

            result.insert(
                result
                    .char_indices()
                    .nth(cursor.char_index + i)
                    .map_or_else(|| result.len(), |(byte_index, _)| byte_index), /* find the utf8 char index of the insert
                                                                                  * in byte index */
                '|',
            );
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
