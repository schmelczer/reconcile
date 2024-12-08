use crate::Token;

/// Given two lists of tokens, returns the offset in the first (old) list from
/// which the two lists have the same tokens until the end of the first list.
/// Thus, the suffix of the old list from the offset to the end is equal to a
/// prefix of the new list.
///
/// If there is no overlap, the function returns the maxmium offset, the length
/// of the old list.
///
/// ## Example
/// ```
/// old: [0, 1, 9, 0, 2, 5]
/// new:       [9, 0, 2, 5, 1]
/// ```
/// > results in an offset of 2
pub fn find_common_overlap<T>(old: &[Token<T>], new: &[Token<T>]) -> usize
where
    T: PartialEq + Clone,
{
    let minimum_offset = old.len().saturating_sub(new.len());
    for offset in minimum_offset..old.len() {
        if old.iter().skip(offset).zip(new.iter()).all(|(a, b)| a == b) {
            return offset;
        }
    }

    old.len()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_common_overlap() {
        assert_eq!(find_common_overlap(&["".into()], &["".into()]), 0);

        assert_eq!(
            find_common_overlap(
                &["a".into(), "b".into(), "c".into()],
                &["b".into(), "c".into(), "a".into()]
            ),
            1
        );

        assert_eq!(
            find_common_overlap(
                &["a".into(), "a".into(), "a".into()],
                &["a".into(), "b".into(), "c".into()]
            ),
            2
        );

        assert_eq!(
            find_common_overlap(
                &["a".into(), "b".into(), "c".into()],
                &["d".into(), "e".into(), "a".into()]
            ),
            3
        );

        assert_eq!(
            find_common_overlap(&["a".into(), "a".into()], &["a".into()]),
            1
        );
    }
}
