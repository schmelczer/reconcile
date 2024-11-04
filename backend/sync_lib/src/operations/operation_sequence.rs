use super::{operation, Operation};
use crate::errors::SyncLibError;
use log::info;
use ropey::Rope;
use similar::utils::diff_graphemes;
use similar::{utils::TextDiffRemapper, ChangeTag, TextDiff};
use similar::{Algorithm, DiffableStrRef};

#[derive(Debug)]
pub struct OperationSequence {
    operations: Vec<Operation>,
}

impl OperationSequence {
    pub fn new(mut operations: Vec<Operation>) -> Self {
        operations.sort();

        Self { operations }
    }

    pub fn try_from_string_diff(
        left: &str,
        right: &str,
        diff_ratio_threshold: f32,
    ) -> Result<Self, SyncLibError> {
        let diff = TextDiff::configure()
            .algorithm(Algorithm::Myers)
            .diff_words(left, right);

        let diff_ratio = 1.0 - diff.ratio();
        if diff_ratio > diff_ratio_threshold {
            return Err(SyncLibError::DiffTooLarge {
                diff_ratio,
                diff_ratio_limit: diff_ratio_threshold as f32,
            });
        }

        let remapper = TextDiffRemapper::from_text_diff(&diff, left, right);

        let mut index = 0;
        let operations = diff
            .ops()
            .iter()
            .flat_map(move |x| remapper.iter_slices(x))
            .map(|(tag, text)| match tag {
                ChangeTag::Equal => {
                    println!("Equal: {}", text);
                    index += text.chars().count();
                    None
                }
                ChangeTag::Insert => {
                    println!("Insert: {}", text);
                    let result = Some(Operation::new(tag, index as u64, text));
                    index += text.chars().count();
                    result
                }
                ChangeTag::Delete => {
                    println!("Delete: {}", text);
                    Some(Operation::new(tag, index as u64, text))
                }
            })
            .flat_map(Option::into_iter)
            .collect::<Vec<_>>();

        Ok(Self::new(operations))
    }

    pub fn apply<'a>(&self, rope_text: &'a mut Rope) -> Result<&'a mut Rope, SyncLibError> {
        for operation in &self.operations {
            println!("Applying operation: {:?}", operation);
            operation.apply(rope_text)?;
            println!("Text after operation: {}", rope_text);
        }

        Ok(rope_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_operations() -> Result<(), SyncLibError> {
        let left = "hello world! How are you?  Adam";
        let right = "Hello, my friend! How are you doing? Albert";

        let operations = OperationSequence::try_from_string_diff(left, right, 0.6)?;

        insta::assert_debug_snapshot!(operations);

        let mut left = Rope::from_str(left);
        let new_right = operations.apply(&mut left)?;

        assert_eq!(new_right.to_string(), right);

        Ok(())
    }

    #[test]
    fn test_calculate_operations_with_large_diff() {
        let left = "hello world! How are you?  Adam";
        let right = "Hello, my friend! How are you doing? Albert";

        let result = OperationSequence::try_from_string_diff(left, right, 0.1);

        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_calculate_operations_with_no_diff() -> Result<(), SyncLibError> {
        let left = "hello world!";
        let right = "hello world!";

        let operations = OperationSequence::try_from_string_diff(left, right, 0.0)?;

        assert_eq!(operations.operations.len(), 0);

        let mut left = Rope::from_str(left);
        let new_right = operations.apply(&mut left)?;

        assert_eq!(new_right.to_string(), right);

        Ok(())
    }
}
