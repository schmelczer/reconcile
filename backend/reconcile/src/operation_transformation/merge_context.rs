use core::fmt::Debug;

use crate::operation_transformation::Operation;

#[derive(Clone, Debug)]
pub struct MergeContext<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    last_operation: Option<Operation<T>>,
    pub shift: i64,
}

impl<T> Default for MergeContext<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    fn default() -> Self {
        MergeContext {
            last_operation: None,
            shift: 0,
        }
    }
}

impl<T> MergeContext<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    pub fn last_operation(&self) -> Option<&Operation<T>> { self.last_operation.as_ref() }

    pub fn replace_last_operation(&mut self, operation: Option<Operation<T>>) {
        self.last_operation = operation;
    }

    /// Replace the last delete operation (if there was one) with a new one
    /// while applying it to the `shift` in case the last operation
    /// was a delete.
    pub fn consume_and_replace_last_operation(&mut self, operation: Option<Operation<T>>) {
        if let Some(Operation::Delete {
            deleted_character_count,
            ..
        }) = self.last_operation.take()
        {
            self.shift -= deleted_character_count as i64;
        }

        self.last_operation = operation;
    }

    /// Remove the last operation (if there was one) in case it is behind the
    /// threshold operation. This updates the `shift` in case the last operation
    /// was a delete.
    pub fn consume_last_operation_if_it_is_too_behind(&mut self, threshold_index: i64) {
        if let Some(last_operation) = self.last_operation.as_ref() {
            if let Operation::Delete {
                deleted_character_count,
                ..
            } = last_operation
            {
                if threshold_index + self.shift > last_operation.end_index() as i64 {
                    self.shift -= *deleted_character_count as i64;
                    self.last_operation = None;
                }
            } else if let Operation::Insert { .. } = last_operation {
                if threshold_index + self.shift - last_operation.len() as i64
                    > last_operation.end_index() as i64
                {
                    self.last_operation = None;
                }
            }
        }
    }
}
