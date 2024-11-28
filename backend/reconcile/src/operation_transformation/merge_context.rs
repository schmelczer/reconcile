use crate::operation_transformation::Operation;

#[derive(Debug, Clone)]
pub struct MergeContext<T>
where
    T: PartialEq + Clone,
{
    pub last_delete: Option<Operation<T>>,
    pub shift: i64,
}

impl<T> Default for MergeContext<T>
where
    T: PartialEq + Clone,
{
    fn default() -> Self {
        MergeContext {
            last_delete: None,
            shift: 0,
        }
    }
}

impl<T> MergeContext<T>
where
    T: PartialEq + Clone,
{
    /// Replace the last delete operation (if there was one) with a new one while
    /// applying it to the shift.
    pub fn replace_delete(&mut self, delete: Option<Operation<T>>) {
        if let Some(produced_last_delete) = self.last_delete.take() {
            self.shift -= produced_last_delete.len() as i64;
        }

        self.last_delete = delete;
    }

    /// Remove the last delete operation (if there was one) in case it is behind the
    /// threshold operation.
    pub fn consume_delete_if_behind_operation(&mut self, threshold_operation: &Operation<T>) {
        match self.last_delete.as_ref() {
            Some(last_delete)
                if threshold_operation.start_index() as i64 + self.shift
                    > last_delete.end_index() as i64 =>
            {
                self.shift -= last_delete.len() as i64;
                self.last_delete = None;
            }
            _ => {}
        }
    }
}
