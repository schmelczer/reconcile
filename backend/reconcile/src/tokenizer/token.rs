use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Token<T>
where
    T: PartialEq + Hash + Clone,
{
    normalised: T,
    original: String,
}

impl<T> Token<T>
where
    T: PartialEq + Hash + Clone,
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
    T: PartialEq + Hash + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.normalised == other.normalised
    }
}

impl<T> Hash for Token<T>
where
    T: PartialEq + Hash + Clone,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.normalised.hash(state);
    }
}
