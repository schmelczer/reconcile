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
    // This might look bad, but this makes sense. The inserts and deltes can be
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
                maybe_previous_delete
                    .take()
                    .into_iter()
                    .chain(maybe_previous_insert.take())
                    .chain(iter::once(next)),
            ) as Box<dyn Iterator<Item = RawOperation<T>>>,
        })
        .collect();

    if let Some(prev) = maybe_previous_delete {
        result.push(prev);
    }

    if let Some(prev) = maybe_previous_insert {
        result.push(prev);
    }

    result
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn test_elongate_operations_empty() {
//         let operations: Vec<RawOperation<()>> = vec![];
//         let result = elongate_operations(operations);
//         assert_eq!(result, vec![]);
//     }

//     #[test]
//     fn test_elongate_operations_single_operation() {
//         let operations = vec![RawOperation::Insert(vec!["test".into()])];
//         let result = elongate_operations(operations);
//         assert_eq!(result.len(), 1);
//         assert!(matches!(result[0], RawOperation::Insert(_)));
//     }

//     #[test]
//     fn test_elongate_operations_interleaved() {
//         let operations = vec![
//             RawOperation::Insert(vec!["a".into()]),
//             RawOperation::Delete(vec!["b".into()]),
//             RawOperation::Insert(vec!["c".into()]),
//             RawOperation::Delete(vec!["d".into()]),
//         ];
//         let result = elongate_operations(operations);
//         assert_eq!(result.len(), 2);
//         assert!(matches!(result[0], RawOperation::Insert(_)));
//         assert!(matches!(result[1], RawOperation::Delete(_)));
//     }

//     #[test]
//     fn test_elongate_operations_with_equal() {
//         let operations = vec![
//             RawOperation::Equal(vec!["a".into()]),
//             RawOperation::Equal(vec!["b".into()]),
//             RawOperation::Insert(vec!["c".into()]),
//             RawOperation::Insert(vec!["d".into()]),
//         ];
//         let result = elongate_operations(operations);
//         assert_eq!(result.len(), 2);
//         assert!(matches!(result[0], RawOperation::Equal(_)));
//         assert!(matches!(result[1], RawOperation::Insert(_)));
//     }

//     #[test]
//     fn test_elongate_operations_mixed_sequence() {
//         let operations = vec![
//             RawOperation::Insert(vec!["a".into()]),
//             RawOperation::Equal(vec!["b".into()]),
//             RawOperation::Delete(vec!["c".into()]),
//             RawOperation::Equal(vec!["d".into()]),
//         ];
//         let result = elongate_operations(operations);
//         assert_eq!(result.len(), 4);
//         assert!(matches!(result[0], RawOperation::Insert(_)));
//         assert!(matches!(result[1], RawOperation::Equal(_)));
//         assert!(matches!(result[2], RawOperation::Delete(_)));
//         assert!(matches!(result[3], RawOperation::Equal(_)));
//     }
// }
