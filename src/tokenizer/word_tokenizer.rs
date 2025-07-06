use super::token::Token;

/// Splits text on word boundaries, creating tokens of alternating words and
/// whitespace with the whitespace getting unique IDs.
///
/// ## Example
///
/// ```not_rust
/// "Hi there!" -> ["Hi", " ", "there!"]
/// ```
pub fn word_tokenizer(text: &str) -> Vec<Token<String>> {
    let mut result = Vec::new();

    let mut previous_boundary_index = 0;
    let mut previous_char_is_whitespace = text.chars().next().is_none_or(char::is_whitespace);

    for (i, c) in text.char_indices() {
        let is_current_char_whitespace = c.is_whitespace();
        if previous_char_is_whitespace != is_current_char_whitespace {
            result.push(text[previous_boundary_index..i].into());
            previous_boundary_index = i;
        }

        previous_char_is_whitespace = is_current_char_whitespace;
    }

    if previous_boundary_index < text.len() {
        result.push(text[previous_boundary_index..].into());
    }

    if result.is_empty() {
        return result;
    }

    // normalize whitespace tokens by concatenating with the following token
    for i in 0..result.len() - 1 {
        if result[i].original().chars().all(char::is_whitespace) {
            let normalized = result[i].normalized().to_owned() + result[i + 1].original();
            result[i].set_normalized(normalized);
        }
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
