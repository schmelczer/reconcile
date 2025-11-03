/// Determine if the given data is a binary or a text file's content.
///
/// Returns the UTF8 parsed string if it's a text, or `None` if it's likely
/// binary.
#[must_use]
pub fn string_or_nothing(data: &[u8]) -> Option<String> {
    if data.contains(&0) {
        // Even though the NUL character is valid in UTF-8, it's highly suspicious in
        // human-readable text.
        return None;
    }

    std::str::from_utf8(data).map(|s| s.to_string()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_or_nothing() {
        assert_eq!(string_or_nothing(&[0, 159, 146, 150]), None);
        assert_eq!(string_or_nothing(&[0, 12]), None);
        assert_eq!(string_or_nothing(b"hello"), Some("hello".into()));
    }
}
