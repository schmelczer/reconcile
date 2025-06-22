use crate::Token;

/// Given two lists of tokens, returns `length` where `old` list somewhere
/// within contains the `length` prefix of the `new` list.
///
/// ## Example
///
/// ```not_rust
/// old: [0, 1, 9, 0, 2, 5]
/// new:       [9, 0, 2, 5, 1]
/// ```
/// > results in a length of 4
///
///
/// ```not_rust
/// old: [0, 1, 9, 0, 2, 5]
/// new:          [0, 2]
/// ```
/// > results in a length of 2
///
/// ```not_rust
/// old: [0, 1, 9, 0, 2, 5]
/// new:          [0, 4]
/// ```
/// > results in a length of 1
pub fn find_longest_prefix_contained_within<T>(old: &[Token<T>], new: &[Token<T>]) -> usize
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    let max_possible = new.len().min(old.len());

    for len in (1..=max_possible).rev() {
        let prefix = &new[..len];
        if old.windows(len).any(|window| window == prefix) {
            return len;
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_common_overlap() {
        assert_eq!(
            find_longest_prefix_contained_within(&["".into()], &["".into()]),
            1
        );

        assert_eq!(
            find_longest_prefix_contained_within(
                &["a".into(), "b".into(), "c".into()],
                &["b".into(), "c".into(), "a".into()]
            ),
            2
        );

        assert_eq!(
            find_longest_prefix_contained_within(
                &["a".into(), "b".into(), "c".into()],
                &["b".into(), "c".into()]
            ),
            2
        );

        assert_eq!(
            find_longest_prefix_contained_within(
                &["a".into(), "b".into(), "c".into()],
                &["b".into()]
            ),
            1
        );

        assert_eq!(
            find_longest_prefix_contained_within(
                &["a".into(), "b".into(), "c".into(), "b".into(), "a".into()],
                &["b".into(), "a".into()]
            ),
            2
        );

        assert_eq!(
            find_longest_prefix_contained_within(
                &["a".into(), "a".into(), "a".into()],
                &["a".into(), "b".into(), "c".into()]
            ),
            1
        );

        assert_eq!(
            find_longest_prefix_contained_within(
                &["a".into(), "b".into(), "c".into()],
                &["d".into(), "e".into(), "a".into()]
            ),
            0
        );
    }
}
