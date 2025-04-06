use super::token::Token;

/// Splits on word boundaries creating alternating words and whitespaces with
/// the whitesspaces getting unique IDs.
///
/// ## Example
///
/// "Hi there!" -> ["Hi", " " ", "there!"]
pub fn word_tokenizer(text: &str) -> Vec<Token<String>> {
    let mut result: Vec<Token<String>> = Vec::new();

    let mut previous_boundary_index = 0;
    let mut previous_char_is_whitespace = text.chars().next().map_or(true, |c| c.is_whitespace());

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

    for i in 0..result.len() - 1 {
        if result[i].original().chars().all(|c| c.is_whitespace()) {
            result[i].normalised = result[i].normalised().to_owned() + result[i + 1].original()
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
