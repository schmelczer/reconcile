#[derive(Debug, Clone)]
pub struct Token<T>
where
    T: PartialEq + Clone,
{
    normalised: T,
    original: String,
}

impl From<&str> for Token<String> {
    fn from(s: &str) -> Self {
        Token {
            normalised: s.to_string(),
            original: s.to_string(),
        }
    }
}

impl<T> Token<T>
where
    T: PartialEq + Clone,
{
    pub fn new(normalised: T, original: String) -> Self {
        Token {
            normalised,
            original,
        }
    }

    pub fn original(&self) -> &str {
        &self.original
    }

    pub fn normalised(&self) -> &T {
        &self.normalised
    }

    pub fn get_original_length(&self) -> usize {
        self.original.chars().count()
    }
}

impl<T> PartialEq for Token<T>
where
    T: PartialEq + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.normalised == other.normalised
    }
}
