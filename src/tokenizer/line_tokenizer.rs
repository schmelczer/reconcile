use super::token::Token;

/// Splits text into lines, preserving line endings as separate tokens.
///
/// ## Example
///
/// ```not_rust
/// "Hello\nWorld!" -> ["Hello", "\n", "World!"]
/// "Line 1\r\nLine 2" -> ["Line 1", "\r\n", "Line 2"]
/// ```
pub fn line_tokenizer(text: &str) -> Vec<Token<String>> {
    let mut result = Vec::new();
    let mut line_start = 0;
    
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '\n' {
            // Add line content if any
            if i > line_start {
                result.push(text[line_start..i].into());
            }
            // Add newline
            result.push("\n".into());
            line_start = i + 1;
        } else if c == '\r' && chars.peek() == Some(&(i + 1, '\n')) {
            // Handle \r\n
            if i > line_start {
                result.push(text[line_start..i].into());
            }
            chars.next(); // consume \n
            result.push("\r\n".into());
            line_start = i + 2;
        }
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
    }
}
