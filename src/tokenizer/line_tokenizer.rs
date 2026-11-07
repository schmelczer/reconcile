use super::token::Token;

/// Splits text into lines, keeping each line ending attached to its content.
/// An unterminated final line is kept as-is, without adding a line ending.
///
/// ## Example
///
/// ```not_rust
/// "Hello\nWorld!" -> ["Hello\n", "World!"]
/// "Line 1\r\nLine 2" -> ["Line 1\r\n", "Line 2"]
/// ```
pub fn line_tokenizer(text: &str) -> Vec<Token<String>> {
    let mut result = Vec::new();
    let mut line_start = 0;

    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let line_end = match c {
            '\r' if chars.peek() == Some(&(i + 1, '\n')) => {
                chars.next(); // consume \n
                i + 2
            }
            '\n' | '\r' => i + 1,
            _ => continue,
        };

        result.push(text[line_start..line_end].into());
        line_start = line_end;
    }

    // Add final line if any
    if line_start < text.len() {
        result.push(text[line_start..].into());
    }

    result
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn test_with_snapshots() {
        assert_debug_snapshot!(line_tokenizer(""));

        assert_debug_snapshot!(line_tokenizer("Hello"));

        assert_debug_snapshot!(line_tokenizer("Hello\nWorld"));

        assert_debug_snapshot!(line_tokenizer("Hello\nWorld\n"));

        assert_debug_snapshot!(line_tokenizer("Line 1\r\nLine 2"));

        assert_debug_snapshot!(line_tokenizer("Multi\nLine\nText\nHere"));

        assert_debug_snapshot!(line_tokenizer("\n"));

        assert_debug_snapshot!(line_tokenizer("\n\n"));

        assert_debug_snapshot!(line_tokenizer("Start\n\nEnd"));

        assert_debug_snapshot!(line_tokenizer("Old\rMac\rStyle"));

        assert_debug_snapshot!(line_tokenizer("Mixed\r\nand\rbare"));

        assert_debug_snapshot!(line_tokenizer("Héllo\r\n世界\r🦊\n\n"));

        assert_debug_snapshot!(line_tokenizer("\r\n\r\r\n\n"));
    }
}
