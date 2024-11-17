#[derive(Debug, Clone)]
pub struct Token {
    pub normalised: String,
    pub original: String,
}

impl Token {
    pub fn new(normalised: String, original: String) -> Self {
        Token {
            normalised,
            original,
        }
    }

    pub fn tokenize(text: &str) -> Vec<Token> {
        text.split_inclusive(|c: char| c.is_whitespace())
            .map(|s| Token::new(s.to_string(), s.to_string()))
            .collect()
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.normalised == other.normalised
    }
}
