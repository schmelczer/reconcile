use super::token::Token;

/// Splits text into UTF-8 characters.
///
/// ```not_rust
/// "Hey!" -> ["H", "e", "y", "!"]
/// ```
pub fn character_tokenizer(text: &str) -> Vec<Token<String>> {
    text.chars()
        .map(|char| Token::new(char.to_string(), char.to_string(), true, true))
        .collect()
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn test_with_snapshots() {
        assert_debug_snapshot!(character_tokenizer(""));

        assert_debug_snapshot!(character_tokenizer(" hello, \nwhere are you?"));
    }
}
