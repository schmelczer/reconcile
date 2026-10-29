use std::{fmt, str::Chars};

/// A helper for building a string sequentially from an original string via
/// insertions, deletions, and copies. All operations use character counts,
/// safe for UTF-8. Methods must be called in-order.
pub struct StringBuilder<'a> {
    original: Chars<'a>,
    buffer: String,

    #[cfg(debug_assertions)]
    position: usize,
}

impl fmt::Debug for StringBuilder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("StringBuilder");
        debug_struct.field("buffer", &self.buffer);

        #[cfg(debug_assertions)]
        debug_struct.field("remaining", &self.original.as_str());

        debug_struct.finish_non_exhaustive()
    }
}

impl StringBuilder<'_> {
    pub fn new(original: &str) -> StringBuilder<'_> {
        StringBuilder {
            original: original.chars(),
            buffer: String::with_capacity(original.len()),

            #[cfg(debug_assertions)]
            position: 0,
        }
    }

    /// Insert a string at the end of the built buffer
    pub fn insert(&mut self, text: &str) {
        self.buffer.push_str(text);
    }

    /// Skip copying `length` characters from the original string to the built
    /// buffer
    pub fn delete(&mut self, length: usize) {
        if length == 0 {
            return;
        }

        self.original.nth(length - 1);

        #[cfg(debug_assertions)]
        {
            self.position += length;
        }
    }

    /// Copy `length` characters from the original string to the built buffer
    pub fn retain(&mut self, length: usize) {
        self.buffer.extend(self.original.by_ref().take(length));

        #[cfg(debug_assertions)]
        {
            self.position += length;
        }
    }

    /// Returns the currently built buffer and clears it to allow consuming
    /// the result incrementally.
    pub fn take(&mut self) -> String {
        std::mem::take(&mut self.buffer)
    }

    /// Check the original-text position and range without copying the text.
    #[cfg(debug_assertions)]
    pub fn assert_range(&self, order: usize, length: usize) {
        debug_assert_eq!(self.position, order, "Operation out of order");
        debug_assert!(
            length == 0 || self.original.clone().nth(length - 1).is_some(),
            "Range out of bounds"
        );
    }

    /// Check the next original characters without allocating a substring.
    #[cfg(debug_assertions)]
    pub fn assert_text(&self, length: usize, text: &str) {
        debug_assert!(
            self.original.clone().take(length).eq(text.chars()),
            "Text `{text}` does not match the next {length} characters of `{}`",
            self.original.as_str()
        );
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_string_builder() {
        let original = "aaa bbb ccc";
        let mut builder = StringBuilder::new(original);

        builder.insert("ddd");
        builder.delete(3);
        builder.retain(8);
        builder.insert(" eee");

        assert_eq!(builder.take(), "ddd bbb ccc eee");

        let original = "abcde";
        let mut builder = StringBuilder::new(original);

        builder.retain(1);
        builder.delete(3);
        builder.retain(1);

        assert_eq!(builder.take(), "ae");
    }

    #[test]
    fn test_empty_original() {
        let original = "";
        let mut builder = StringBuilder::new(original);

        builder.insert("test");
        assert_eq!(builder.take(), "test");
    }

    #[test]
    fn test_unicode_characters() {
        let original = "こんにちは";
        let mut builder = StringBuilder::new(original);

        builder.retain(3);
        builder.insert("世界, "); // Insert "World, "
        builder.retain(2);

        assert_eq!(builder.take(), "こんに世界, ちは");
    }

    #[test]
    fn test_retain_all() {
        let original = "Hello, world!";
        let mut builder = StringBuilder::new(original);

        builder.retain(original.len());
        assert_eq!(builder.take(), original);
    }

    #[test]
    fn test_delete_all() {
        let original = "Hello";
        let mut builder = StringBuilder::new(original);

        builder.delete(original.len());
        builder.insert("Hi");
        assert_eq!(builder.take(), "Hi");
    }
}
