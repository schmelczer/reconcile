use std::ops::{Index, Range};

/// Given two lookups and ranges calculates the length of the common prefix.
/// Copied from https://github.com/mitsuhiko/similar/blob/7e15c44de11a1cd61e1149189929e189ef977fd8/src/algorithms/utils.rs
pub fn common_prefix_len<Old, New>(
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
        .zip(old_range)
        .take_while(|x| new[x.0] == old[x.1])
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_common_prefix_len() {
        assert_eq!(
            common_prefix_len("".as_bytes(), 0..0, "".as_bytes(), 0..0),
            0
        );
        assert_eq!(
            common_prefix_len("foobarbaz".as_bytes(), 0..9, "foobarblah".as_bytes(), 0..10),
            7
        );
        assert_eq!(
            common_prefix_len("foobarbaz".as_bytes(), 0..9, "blablabla".as_bytes(), 0..9),
            0
        );
        assert_eq!(
            common_prefix_len("foobarbaz".as_bytes(), 3..9, "foobarblah".as_bytes(), 3..10),
            4
        );
    }
}
