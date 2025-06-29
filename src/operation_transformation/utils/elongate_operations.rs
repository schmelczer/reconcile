use core::iter;
use std::fmt::Debug;

use crate::raw_operation::RawOperation;

/// Elongates the operations by merging adjacent insertions and deletions that
/// can be joined. This makes the subsequent merging of operations more
/// intuitive.
pub fn elongate_operations<I, T>(raw_operations: I) -> Vec<RawOperation<T>>
where
    I: IntoIterator<Item = RawOperation<T>>,
    T: PartialEq + Clone + Debug,
{
    // This might look bad, but this makes sense. The inserts and deltes can be
    // interleaved, such as: IDIDID and we need to turn this into IIIDDD.
    // So we need to keep track of both the last insert and delete operations, not
    // just the last one.
    let mut maybe_previous_insert: Option<RawOperation<T>> = None;
    let mut maybe_previous_delete: Option<RawOperation<T>> = None;

    // We don't elongate `equals` as they're needed to maintain cursor positions
    // when merging against deletes.
    let mut result: Vec<RawOperation<T>> = raw_operations
        .into_iter()
        .flat_map(|next| match next {
            RawOperation::Insert(..) => match maybe_previous_insert.take() {
                Some(prev) if prev.is_right_joinable() && next.is_left_joinable() => {
                    maybe_previous_insert = Some(prev.join(next));
                    Box::new(iter::empty()) as Box<dyn Iterator<Item = RawOperation<T>>>
                }
                prev => {
                    maybe_previous_insert = Some(next);
                    Box::new(prev.into_iter())
                }
            },
            RawOperation::Delete(..) => match maybe_previous_delete.take() {
                Some(prev) if prev.is_right_joinable() && next.is_left_joinable() => {
                    maybe_previous_delete = Some(prev.join(next));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::token::Token;

    // Helper constructors for cleaner tests
    fn ins(texts: &[&str]) -> RawOperation<String> {
        RawOperation::Insert(texts.iter().map(|t| Token::from(*t)).collect())
    }

    fn del(texts: &[&str]) -> RawOperation<String> {
        RawOperation::Delete(texts.iter().map(|t| Token::from(*t)).collect())
    }

    fn ins_custom(text: &str, lj: bool, rj: bool) -> RawOperation<String> {
        RawOperation::Insert(vec![Token::new(text.to_owned(), text.to_owned(), lj, rj)])
    }

    #[test]
    fn merges_adjacent_joinable_inserts() {
        let ops = vec![ins(&["a"]), ins(&["b"]), ins(&["c"])];
        let result = elongate_operations(ops);
        assert_eq!(result.len(), 1);
        match &result[0] {
            RawOperation::Insert(tokens) => {
                let originals: String = tokens
                    .iter()
                    .map(crate::tokenizer::token::Token::original)
                    .collect();
                assert_eq!(originals, "abc");
            }
            _ => panic!("Expected single Insert operation"),
        }
    }

    #[test]
    fn does_not_merge_when_not_joinable() {
        let ops = vec![
            ins_custom("a", true, false), // not right-joinable
            ins_custom("b", true, true),  // left-joinable but previous isn't right-joinable
        ];
        let result = elongate_operations(ops);
        assert_eq!(
            result.len(),
            2,
            "Operations should remain separate when not joinable"
        );
    }

    #[test]
    fn merges_interleaved_insert_delete_sequences() {
        // Pattern IDID -> II DD
        let ops = vec![ins(&["i1"]), del(&["d1"]), ins(&["i2"]), del(&["d2"])];
        let result = elongate_operations(ops);

        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], RawOperation::Delete(_)));
        assert!(matches!(result[1], RawOperation::Insert(_)));
    }
}
