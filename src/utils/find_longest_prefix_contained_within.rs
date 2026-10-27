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
pub fn find_longest_prefix_contained_within<T>(old: &[Token<T>], new: &[Token<T>]) -> Range<usize>
where
    T: PartialEq + Clone + Debug,
{
    let max_possible = new.len().min(old.len());

    for len in (1..=max_possible).rev() {
        let prefix = &new[..len];
        if let Some(start) = old.windows(len).position(|window| window == prefix) {
            return start..start + len;
        }
    }

    0..0
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
}
