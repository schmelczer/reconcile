use std::iter::Iterator;

/// A helper for building a string in-order based on an original string and a
/// series of insertions, deletions, and copies applied to it. It is safe to use
/// with UTF-8 strings as all operations are based on character indices. The
/// methods must be called in-order.
pub struct StringBuilder<'a> {
    original: Box<dyn Iterator<Item = char> + 'a>,
    buffer: String,

    #[cfg(debug_assertions)]
    remaining: String,
}

impl StringBuilder<'_> {
    pub fn new(original: &str) -> StringBuilder<'_> {
        StringBuilder {
            original: Box::new(original.chars()),
            buffer: String::with_capacity(original.len()),

            #[cfg(debug_assertions)]
            remaining: original.to_owned(),
        }
    }

    /// Insert a string at the end of the built buffer.
    pub fn insert(&mut self, text: &str) { self.buffer.push_str(text); }

    /// Skip copying `length` characters from the original string to the built
    /// buffer.
    pub fn delete(&mut self, length: usize) {
        if length == 0 {
            return;
        }

        self.original.nth(length - 1);

        #[cfg(debug_assertions)]
        {
            self.remaining = self.remaining.chars().skip(length).collect();
        }
    }

    /// Copy `length` characters from the original string to the built buffer.
    pub fn retain(&mut self, length: usize) {
        self.buffer.extend(self.original.by_ref().take(length));

        #[cfg(debug_assertions)]
        {
            self.remaining = self.remaining.chars().skip(length).collect();
        }
    }

    /// Returns the currently built buffer and clears it to allow consuming
    /// the result incrementally.
    pub fn take(&mut self) -> String { std::mem::take(&mut self.buffer) }

    /// Get a slice of the remaining original string. The slice starts from
    /// where the next delete/retain operation would start and is of length
    /// `length`.
    #[cfg(debug_assertions)]
    pub fn get_slice_from_remaining(&self, length: usize) -> String {
        let result = self.remaining.chars().take(length).collect::<String>();

        debug_assert_eq!(result.chars().count(), length, "Range out of bounds");

        result
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
    fn test_get_slice() {
        let original = "abcdef";
        let builder = StringBuilder::new(original);

        // Test getting a slice of the original string
        assert_eq!(builder.get_slice_from_remaining(3), "abc");

        // Test getting a slice that includes both buffer and remaining original
        let mut builder = StringBuilder::new(original);
        builder.retain(2); // "ab" in buffer
        assert_eq!(builder.get_slice_from_remaining(2), "cd");
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
