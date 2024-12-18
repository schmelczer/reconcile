use core::ops::{Index, Range};

/// Given two lookups and ranges calculates the length of common suffix.
/// Copied from <https://github.com/mitsuhiko/similar/blob/7e15c44de11a1cd61e1149189929e189ef977fd8/src/algorithms/utils.rs>
pub fn common_suffix_len<Old, New>(
    old: &Old,
    old_range: Range<usize>,
    new: &New,
    new_range: Range<usize>,
) -> usize
where
    Old: Index<usize> + ?Sized,
    New: Index<usize> + ?Sized,
    New::Output: PartialEq<Old::Output>,
{
    new_range
        .rev()
        .zip(old_range.rev())
        .take_while(|x| new[x.0] == old[x.1])
        .count()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_common_suffix_len() {
        assert_eq!(
            common_suffix_len("".as_bytes(), 0..0, "".as_bytes(), 0..0),
            0
        );
        assert_eq!(
            common_suffix_len("1234".as_bytes(), 0..4, "X0001234".as_bytes(), 0..8),
            4
        );
        assert_eq!(
            common_suffix_len("1234".as_bytes(), 0..4, "Xxxx".as_bytes(), 0..4),
            0
        );
        assert_eq!(
            common_suffix_len("1234".as_bytes(), 2..4, "01234".as_bytes(), 2..5),
            2
        );
    }
}
