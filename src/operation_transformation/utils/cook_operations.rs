use crate::{
    diffs::raw_operation::RawOperation, operation_transformation::Operation, utils::side::Side,
};

/// Turn raw operations into ordered operations while keeping track of the
/// original token's indexes.
pub fn cook_operations<I, T>(raw_operations: I, side: Side) -> impl Iterator<Item = Operation<T>>
where
    I: IntoIterator<Item = RawOperation<T>>,
    T: PartialEq + Clone + std::fmt::Debug,
{
    let mut original_text_index = 0; // this is the start index of the operation on the original text

    raw_operations.into_iter().map(move |raw_operation| {
        let length = raw_operation.original_text_length();

        match raw_operation {
            RawOperation::Equal(..) => {
                let op = if cfg!(debug_assertions) {
                    Operation::create_equal_with_text(
                        original_text_index,
                        raw_operation.get_original_text(),
                    )
                } else {
                    Operation::create_equal(original_text_index, length)
                };

                original_text_index += length;

                op
            }
            RawOperation::Insert(tokens) => {
                Operation::create_insert(original_text_index, tokens, side)
            }
            RawOperation::Delete(..) => {
                let op = if cfg!(debug_assertions) {
                    Operation::create_delete_with_text(
                        original_text_index,
                        raw_operation.get_original_text(),
                        side,
                    )
                } else {
                    Operation::create_delete(original_text_index, length, side)
                };

                original_text_index += length;

                op
            }
        }
    })
}
