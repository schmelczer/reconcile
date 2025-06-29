/// Heuristically determine if the given data is a binary or a text file's
/// content.
#[must_use]
pub fn is_binary(data: &[u8]) -> bool {
    if data.contains(&0) {
        // Even though the NUL character is valid in UTF-8, it's highly suspicious in
        // human-readable text.
        return true;
    }

    std::str::from_utf8(data).is_err()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_binary() {
        assert!(is_binary(&[0, 159, 146, 150]));
        assert!(is_binary(&[0, 12]));
        assert!(!is_binary(b"hello"));
    }
}
