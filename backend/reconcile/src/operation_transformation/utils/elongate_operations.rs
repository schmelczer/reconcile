use core::iter;

use crate::diffs::raw_operation::RawOperation;

/// Elongates the operations by merging adjacent insertions and deletions that
/// can be joined. This makes the subsequent merging of operations more
/// intuitive.
pub fn elongate_operations<I, T>(raw_operations: I) -> Vec<RawOperation<T>>
where
    I: IntoIterator<Item = RawOperation<T>>,
    T: PartialEq + Clone + std::fmt::Debug,
{
    // This might look bad, but this makes sense. The inserts and deletes can be
    // interleaved, such as: IDIDID and we need to turn this into IIIDDD.
    // So we need to keep track of both the last insert and delete operations, not
    // just the last one.
    let mut maybe_previous_insert: Option<RawOperation<T>> = None;
    let mut maybe_previous_delete: Option<RawOperation<T>> = None;

    let mut result: Vec<RawOperation<T>> = raw_operations
        .into_iter()
        .flat_map(|next| match next {
            RawOperation::Insert(..) => match maybe_previous_insert.take() {
                Some(prev) if prev.is_right_joinable() && next.is_left_joinable() => {
                    maybe_previous_insert = Some(prev.extend(next));
                    Box::new(iter::empty()) as Box<dyn Iterator<Item = RawOperation<T>>>
                }
                prev => {
                    maybe_previous_insert = Some(next);
                    Box::new(prev.into_iter())
                }
            },
            RawOperation::Delete(..) => match maybe_previous_delete.take() {
                Some(prev) if prev.is_right_joinable() && next.is_left_joinable() => {
                    maybe_previous_delete = Some(prev.extend(next));
                    Box::new(iter::empty()) as Box<dyn Iterator<Item = RawOperation<T>>>
                }
                prev => {
                    maybe_previous_delete = Some(next);
                    Box::new(prev.into_iter())
                }
            },
            RawOperation::Equal(..) => Box::new(
                maybe_previous_insert
                    .take()
                    .into_iter()
                    .chain(maybe_previous_delete.take())
                    .chain(iter::once(next)),
            ) as Box<dyn Iterator<Item = RawOperation<T>>>,
        })
        .collect();

    if let Some(prev) = maybe_previous_insert {
        result.push(prev);
    }

    if let Some(prev) = maybe_previous_delete {
        result.push(prev);
    }

    result
}
