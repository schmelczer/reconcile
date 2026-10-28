use std::{fmt::Debug, ops::Range};

use crate::Token;

/// Returns the range in `old` containing the longest prefix of `new`.
/// Uses the first occurrence when that prefix appears more than once,
/// or `0..0` if there is no overlap.
///
/// ## Example
///
/// ```not_rust
/// old: [0, 1, 9, 0, 2, 5]
/// new:       [9, 0, 2, 5, 1]
/// ```
/// > results in a range of 2..6
///
///
/// ```not_rust
/// old: [0, 1, 9, 0, 2, 5]
/// new:          [0, 2]
/// ```
/// > results in a range of 3..5
///
/// ```not_rust
/// old: [0, 1, 9, 0, 2, 5]
/// new: [0, 4]
/// ```
/// > results in a range of 0..1
///
/// Runs in `O(old.len() + new.len())` using the Knuth-Morris-Pratt algorithm.
pub fn find_longest_prefix_contained_within<T>(old: &[Token<T>], new: &[Token<T>]) -> Range<usize>
where
    T: PartialEq + Clone + Debug,
{
    if new.is_empty() {
        return 0..0;
    }

    // `fallback[i]` is the length of the longest proper prefix of `new[..=i]`
    // that is also a suffix of it
    let mut fallback = vec![0; new.len()];
    let mut matched = 0;
    for i in 1..new.len() {
        while matched > 0 && new[i] != new[matched] {
            matched = fallback[matched - 1];
        }
        if new[i] == new[matched] {
            matched += 1;
        }
        fallback[i] = matched;
    }

    // `matched` is the longest prefix of `new` ending at the current token of
    // `old`, so it first reaches its maximum at the earliest occurrence
    let mut longest = 0..0;
    matched = 0;
    for (i, token) in old.iter().enumerate() {
        while matched > 0 && *token != new[matched] {
            matched = fallback[matched - 1];
        }
        if *token == new[matched] {
            matched += 1;
        }
        if matched > longest.len() {
            longest = i + 1 - matched..i + 1;
            if matched == new.len() {
                break;
            }
        }
    }

    longest
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_common_overlap() {
        assert_eq!(
            find_longest_prefix_contained_within(&["".into()], &["".into()]),
            0..1
        );

        assert_eq!(
            find_longest_prefix_contained_within(
                &["a".into(), "b".into(), "c".into()],
                &["b".into(), "c".into(), "a".into()]
            ),
            1..3
        );

        assert_eq!(
            find_longest_prefix_contained_within(
                &["a".into(), "b".into(), "c".into()],
                &["b".into(), "c".into()]
            ),
            1..3
        );

        assert_eq!(
            find_longest_prefix_contained_within(
                &["a".into(), "b".into(), "c".into()],
                &["b".into()]
            ),
            1..2
        );

        assert_eq!(
            find_longest_prefix_contained_within(
                &["a".into(), "b".into(), "c".into(), "b".into(), "a".into()],
                &["b".into(), "a".into()]
            ),
            3..5
        );

        assert_eq!(
            find_longest_prefix_contained_within(
                &["a".into(), "a".into(), "a".into()],
                &["a".into(), "b".into(), "c".into()]
            ),
            0..1
        );

        assert_eq!(
            find_longest_prefix_contained_within(
                &["a".into(), "b".into(), "a".into(), "b".into()],
                &["a".into(), "b".into()]
            ),
            0..2
        );

        assert_eq!(
            find_longest_prefix_contained_within(
                &["a".into(), "b".into(), "c".into()],
                &["d".into(), "e".into(), "a".into()]
            ),
            0..0
        );
    }

    #[test]
    fn test_matches_brute_force_on_short_sequences() {
        fn brute_force(old: &[Token<String>], new: &[Token<String>]) -> Range<usize> {
            for len in (1..=new.len().min(old.len())).rev() {
                if let Some(start) = old.windows(len).position(|window| window == &new[..len]) {
                    return start..start + len;
                }
            }
            0..0
        }

        let mut sequences: Vec<Vec<Token<String>>> = vec![Vec::new()];
        let mut level = sequences.clone();
        for _ in 0..5 {
            level = level
                .iter()
                .flat_map(|sequence| {
                    ["a", "b", "c"].map(|token| {
                        let mut extended = sequence.clone();
                        extended.push(token.into());
                        extended
                    })
                })
                .collect();
            sequences.extend(level.iter().cloned());
        }

        for old in &sequences {
            for new in &sequences {
                assert_eq!(
                    find_longest_prefix_contained_within(old, new),
                    brute_force(old, new),
                    "old={old:?}, new={new:?}"
                );
            }
        }
    }
}
