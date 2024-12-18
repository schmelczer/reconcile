use core::ops::Range;

/// A helper for building a string in order based on an original string and a
/// series of insertions and deletions applied to it. It is safe to use with
/// UTF-8 strings as all operations are based on character indices.
#[derive(Debug, Clone)]
pub struct StringBuilder<'a> {
    original: &'a str,
    last_old_char_index: usize,
    buffer: String,
}

impl StringBuilder<'_> {
    pub fn new(original: &str) -> StringBuilder {
        StringBuilder {
            original,
            last_old_char_index: 0,
            buffer: String::with_capacity(original.len()),
        }
    }

    /// Insert a string at the given index after copying the original string up
    /// to that index from the last insertion or deletion.
    pub fn insert(&mut self, from: usize, text: &str) {
        self.copy_until(from);
        self.buffer.push_str(text);
    }

    /// Delete a string at the given index after copying the original string up
    /// to that index from the last insertion or deletion.
    pub fn delete(&mut self, range: core::ops::Range<usize>) {
        self.copy_until(range.start);
        self.last_old_char_index += range.len();
    }

    fn copy_until(&mut self, index: usize) {
        let current_char_count = self.buffer.chars().count();
        debug_assert!(
            index >= current_char_count,
            "String builder only support building in order"
        );

        let jump = index - current_char_count;

        self.buffer.push_str(
            &self
                .original
                .chars()
                .skip(self.last_old_char_index)
                .take(jump)
                .collect::<String>(),
        );
        self.last_old_char_index += jump;
    }

    /// Finish building the string after copying the remaining original string
    /// since the last insertion or deletion.
    pub fn build(mut self) -> String {
        self.buffer.push_str(
            &self
                .original
                .chars()
                .skip(self.last_old_char_index)
                .collect::<String>(),
        );

        self.buffer
    }

    pub fn get_slice(&self, range: Range<usize>) -> String {
        let result = self
            .buffer
            .chars()
            .chain(self.original.chars().skip(self.last_old_char_index))
            .skip(range.start)
            .take(range.end - range.start)
            .collect::<String>();

        debug_assert_eq!(result.chars().count(), range.len(), "Range out of bounds",);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_builder() {
        let original = "aaa bbb ccc";
        let mut builder = StringBuilder::new(original);

        builder.insert(0, "ddd ");
        builder.delete(4..8);
        builder.insert(11, " eee");

        assert_eq!(builder.build(), "ddd bbb ccc eee");
    }

    #[test]
    fn test_string_builder2() {
        let original = "abcde";
        let mut builder = StringBuilder::new(original);

        builder.delete(1..4);

        assert_eq!(builder.build(), "ae");
    }
}
