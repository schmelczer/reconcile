use crate::{
    diffs::raw_operation::RawOperation,
    operation_transformation::{Operation, ordered_operation::OrderedOperation},
};

/// Turn raw operations into ordered operations while keeping track of indexes.
pub fn cook_operations<I, T>(raw_operations: I) -> impl Iterator<Item = OrderedOperation<T>>
where
    I: IntoIterator<Item = RawOperation<T>>,
    T: PartialEq + Clone + std::fmt::Debug,
{
    let mut order = 0; // this is the start index of the operation on the original text

    raw_operations.into_iter().filter_map(move |raw_operation| {
        let length = raw_operation.original_text_length();

        match raw_operation {
            RawOperation::Equal(..) => {
                let op = if cfg!(debug_assertions) {
                    Operation::create_equal_with_text(raw_operation.get_original_text())
                } else {
                    Operation::create_equal(length)
                }
                .map(|operation| OrderedOperation { order, operation });

                order += length;

                op
            }
            RawOperation::Insert(tokens) => Operation::create_insert(tokens)
                .map(|operation| OrderedOperation { order, operation }),
            RawOperation::Delete(..) => {
                let op = if cfg!(debug_assertions) {
                    Operation::create_delete_with_text(raw_operation.get_original_text())
                } else {
                    Operation::create_delete(length)
                }
                .map(|operation| OrderedOperation { order, operation });

                order += length;

                op
            }
        }
    })
}
