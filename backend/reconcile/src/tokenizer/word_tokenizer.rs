use super::token::Token;

pub fn word_tokenizer(text: &str) -> Vec<Token<String>> {
    text.split_inclusive(char::is_whitespace)
        .map(|s| Token::new(s.to_string(), s.to_string()))
        .collect()
}
