use super::token::Token;

/// Splits on whitespace keeping the leading whitespace.
///
///     
/// ## Example
///
/// "Hi there!" -> ["Hi", " there!"]
pub fn word_tokenizer(text: &str) -> Vec<Token<String>> {
    let mut result: Vec<Token<String>> = Vec::new();

    let mut last_whitespace = 0;
    let mut previous_char_is_whitespace = true;

    for (i, c) in text.char_indices() {
        let is_current_char_whitespace = c.is_whitespace();
        if !previous_char_is_whitespace && is_current_char_whitespace {
            result.push(text[last_whitespace..i].into());
            last_whitespace = i;
        }

        previous_char_is_whitespace = is_current_char_whitespace;
    }

    if last_whitespace < text.len() {
        result.push(text[last_whitespace..].into());
    }

    result
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn test_with_snapshots() {
        assert_debug_snapshot!(word_tokenizer("Hi there!"));

        assert_debug_snapshot!(word_tokenizer(""));

        assert_debug_snapshot!(word_tokenizer(" what? "));

        assert_debug_snapshot!(word_tokenizer(" hello, \nwhere are you?"));
    }
}
