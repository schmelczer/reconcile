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
    let mut inserts: Vec<RawOperation<T>> = Vec::new();
    let mut deletes: Vec<RawOperation<T>> = Vec::new();
    let mut result = Vec::new();

    // Emit all deletions before insertions within each changed span, even
    // when tokens cannot be joined. Otherwise an insertion followed by a
    // deletion at the same offset can separate identical insertions during
    // merging and prevent their deduplication.
    for next in raw_operations {
        let pending = match next {
            RawOperation::Insert(..) => &mut inserts,
            RawOperation::Delete(..) => &mut deletes,
            RawOperation::Equal(..) => {
                result.append(&mut deletes);
                result.append(&mut inserts);

                // Keep retains separate for cursor positioning
                result.push(next);

                continue;
            }
        };

        match pending.pop() {
            Some(prev) if prev.is_right_joinable() && next.is_left_joinable() => {
                pending.push(prev.join(next));
            }
            Some(prev) => pending.extend([prev, next]),
            None => pending.push(next),
        }
    }

    result.append(&mut deletes);
    result.append(&mut inserts);

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
        // Pattern IDID -> DD II
        let ops = vec![ins(&["i1"]), del(&["d1"]), ins(&["i2"]), del(&["d2"])];
        let result = elongate_operations(ops);

        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], RawOperation::Delete(_)));
        assert!(matches!(result[1], RawOperation::Insert(_)));
    }

    #[test]
    fn orders_non_joinable_edits_before_each_retain() {
        let insert1 = ins_custom("a", false, false);
        let insert2 = ins_custom("\n", false, false);
        let delete1 = RawOperation::Delete(vec![Token::new(
            "b".to_owned(),
            "b".to_owned(),
            false,
            false,
        )]);
        let delete2 = del(&["c"]);
        let retain = RawOperation::Equal(vec!["d".into()]);
        let ops = vec![
            insert1.clone(),
            delete1.clone(),
            insert2.clone(),
            delete2.clone(),
            retain.clone(),
            insert1.clone(),
            delete1.clone(),
        ];

        assert_eq!(
            elongate_operations(ops),
            vec![
                delete1.clone(),
                delete2,
                insert1.clone(),
                insert2,
                retain,
                delete1,
                insert1,
            ],
        );
    }
}
