//! LCS diff algorithm.
//!
//! * time: `O((NM)D log (M)D)`
//! * space `O(MN)`
use std::collections::BTreeMap;
use std::ops::{Index, Range};

use crate::tokenizer::token::Token;

use super::raw_operation::RawOperation;
use super::utils::{common_prefix_len, common_suffix_len};

/// LCS diff algorithm.
/// Copied from https://github.com/mitsuhiko/similar/blob/7e15c44de11a1cd61e1149189929e189ef977fd8/src/algorithms/lcs.rs
pub fn diff(old: &[Token], new: &[Token]) -> Vec<RawOperation> {
    let common_prefix_len = common_prefix_len(old, 0..old.len(), new, 0..new.len());
    let common_suffix_len = common_suffix_len(
        old,
        common_prefix_len..old.len(),
        new,
        common_prefix_len..new.len(),
    );

    let maybe_table = make_table(
        old,
        common_prefix_len..(old.len() - common_suffix_len),
        new,
        common_prefix_len..(new.len() - common_suffix_len),
    );
    let mut old_idx = 0;
    let mut new_idx = 0;
    let new_len = new.len() - common_prefix_len - common_suffix_len;
    let old_len = old.len() - common_prefix_len - common_suffix_len;

    let mut result: Vec<RawOperation> = Vec::new();
    if common_prefix_len > 0 {
        result.push(RawOperation::Equal(old[0..common_prefix_len].to_vec()));
    }

    if let Some(table) = maybe_table {
        while new_idx < new_len && old_idx < old_len {
            let old_orig_idx = common_prefix_len + old_idx;
            let new_orig_idx = common_prefix_len + new_idx;

            if new[new_orig_idx] == old[old_orig_idx] {
                result.push(RawOperation::Equal(vec![old[old_orig_idx].clone()]));
                old_idx += 1;
                new_idx += 1;
            } else if table.get(&(new_idx, old_idx + 1)).unwrap_or(&0)
                >= table.get(&(new_idx + 1, old_idx)).unwrap_or(&0)
            {
                result.push(RawOperation::Delete(vec![old[old_orig_idx].clone()]));
                old_idx += 1;
            } else {
                result.push(RawOperation::Insert(vec![new[new_orig_idx].clone()]));
                new_idx += 1;
            }
        }
    } else {
        let old_orig_idx = common_prefix_len + old_idx;
        let new_orig_idx = common_prefix_len + new_idx;

        result.push(RawOperation::Delete(
            old[old_orig_idx..old_orig_idx + old_len].to_vec(),
        ));
        result.push(RawOperation::Insert(
            new[new_orig_idx..new_orig_idx + new_len].to_vec(),
        ));
    }

    if old_idx < old_len {
        result.push(RawOperation::Delete(
            old[common_prefix_len + old_idx..common_prefix_len + old_len].to_vec(),
        ));
        old_idx += old_len - old_idx;
    }

    if new_idx < new_len {
        result.push(RawOperation::Insert(
            new[common_prefix_len + new_idx..common_prefix_len + new_len].to_vec(),
        ));
    }

    if common_suffix_len > 0 {
        result.push(RawOperation::Equal(
            old[old_len + common_prefix_len..old_len + common_prefix_len + common_suffix_len]
                .to_vec(),
        ));
    }

    result
}

fn make_table<Old, New>(
    old: &Old,
    old_range: Range<usize>,
    new: &New,
    new_range: Range<usize>,
) -> Option<BTreeMap<(usize, usize), u32>>
where
    Old: Index<usize> + ?Sized,
    New: Index<usize> + ?Sized,
    New::Output: PartialEq<Old::Output>,
{
    let old_len = old_range.len();
    let new_len = new_range.len();
    let mut table = BTreeMap::new();

    for i in (0..new_len).rev() {
        for j in (0..old_len).rev() {
            let val = if new[i] == old[j] {
                table.get(&(i + 1, j + 1)).unwrap_or(&0) + 1
            } else {
                *table
                    .get(&(i + 1, j))
                    .unwrap_or(&0)
                    .max(table.get(&(i, j + 1)).unwrap_or(&0))
            };
            if val > 0 {
                table.insert((i, j), val);
            }
        }
    }

    Some(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;

    #[test]
    fn test_table() {
        let table = make_table(&vec![2, 3], 0..2, &vec![0, 1, 2], 0..3).unwrap();
        let expected = {
            let mut m = BTreeMap::new();
            m.insert((1, 0), 1);
            m.insert((0, 0), 1);
            m.insert((2, 0), 1);
            m
        };
        assert_eq!(table, expected);
    }

    #[test]
    fn test_empty_examples() {
        assert_eq!(diff(&[], &[]), vec![]);
        assert_eq!(
            diff(&[Token::new("a".to_string(), "a".to_string())], &[]),
            vec![RawOperation::Delete(vec![Token::new(
                "a".to_string(),
                "a".to_string()
            )])]
        );
        assert_eq!(
            diff(&[], &[Token::new("a".to_string(), "a".to_string())]),
            vec![RawOperation::Insert(vec![Token::new(
                "a".to_string(),
                "a".to_string()
            )])]
        );
    }
}
