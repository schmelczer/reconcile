//! Taken from <https://github.com/mitsuhiko/similar/blob/7e15c44de11a1cd61e1149189929e189ef977fd8/src/algorithms/myers.rs>
//!
//! Myers' diff algorithm.
//!
//! * time: `O((N+M)D)`
//! * space `O(N+M)`
//!
//! See [the original article by Eugene W. Myers](http://www.xmailserver.org/diff2.pdf)
//! describing it.
//!
//! The implementation of this algorithm is based on the implementation by
//! Brandon Williams.
//!
//! # Complexity
//!
//! The worst case (completely dissimilar inputs) is `O((N+M)²)` time. In
//! practice the divide-and-conquer strategy with prefix/suffix stripping keeps
//! subproblems small for typical text.

use std::{
    fmt::Debug,
    ops::{Index, IndexMut, Range},
    vec,
};

use crate::{
    raw_operation::RawOperation,
    tokenizer::token::Token,
    utils::{common_prefix_len::common_prefix_len, common_suffix_len::common_suffix_len},
};

/// Myers' diff algorithm.
///
/// Diff `old`, between indices `old_range` and `new` between indices
/// `new_range`.
///
/// The returned `RawOperations` each wrap a single token.
pub fn myers_diff<T>(old: &[Token<T>], new: &[Token<T>]) -> Vec<RawOperation<T>>
where
    T: PartialEq + Clone + Debug,
{
    let max_edit_distance = (old.len() + new.len()).div_ceil(2) + 1;
    let mut backward_endpoints = FurthestEndpoints::new(max_edit_distance);
    let mut forward_endpoints = FurthestEndpoints::new(max_edit_distance);
    let mut result = Vec::with_capacity(old.len() + new.len());

    conquer(
        old,
        0..old.len(),
        new,
        0..new.len(),
        &mut forward_endpoints,
        &mut backward_endpoints,
        &mut result,
    );

    result
}

// A D-path is a path which starts at (0,0) that has exactly D non-diagonal
// edges. All D-paths consist of a (D - 1)-path followed by a non-diagonal edge
// and then a possibly empty sequence of diagonal edges called a snake.

/// Contains the endpoints of the furthest reaching `D-paths`. For each
/// recorded endpoint `(x, y)` on diagonal `k`, we only need to retain `x`
/// because `y` can be computed from `x - k`. In other words, this is an array
/// of integers where `endpoints[k]` contains the row index of the endpoint of
/// the furthest reaching path on diagonal `k`.
///
/// We can't use a traditional Vec since we use `k` as an index and it can take
/// on negative values. So instead this is a light-weight wrapper around a Vec
/// plus an `offset` which is the maximum value `k` can take on, used to map
/// negative `k`'s back to a value >= 0.
#[derive(Debug)]
struct FurthestEndpoints {
    offset: isize,
    endpoints: Vec<usize>,
}

impl FurthestEndpoints {
    fn new(max_edit_distance: usize) -> Self {
        let offset =
            isize::try_from(max_edit_distance).expect("max_edit_distance must fit in isize");
        Self {
            offset,
            endpoints: vec![0; 2 * max_edit_distance + 1],
        }
    }

    fn len(&self) -> usize {
        self.endpoints.len()
    }
}

impl Index<isize> for FurthestEndpoints {
    type Output = usize;

    fn index(&self, diagonal: isize) -> &Self::Output {
        let idx =
            usize::try_from(diagonal + self.offset).expect("diagonal + offset must fit in usize");
        &self.endpoints[idx]
    }
}

impl IndexMut<isize> for FurthestEndpoints {
    fn index_mut(&mut self, diagonal: isize) -> &mut Self::Output {
        let idx =
            usize::try_from(diagonal + self.offset).expect("diagonal + offset must fit in usize");
        &mut self.endpoints[idx]
    }
}

fn split_at(range: Range<usize>, at: usize) -> (Range<usize>, Range<usize>) {
    (range.start..at, at..range.end)
}

/// Adjust a lower diagonal bound so it has the same parity as `edit_distance`.
/// Diagonals are visited in steps of 2, so `lower` must share `edit_distance`'s
/// parity.
fn align_lower_bound(lower: isize, edit_distance: isize) -> isize {
    if (lower & 1) == (edit_distance & 1) {
        lower
    } else {
        lower + 1
    }
}

/// Adjust an upper diagonal bound so it has the same parity as `edit_distance`.
fn align_upper_bound(upper: isize, edit_distance: isize) -> isize {
    if (upper & 1) == (edit_distance & 1) {
        upper
    } else {
        upper - 1
    }
}

/// A `Snake` is a sequence of diagonal edges in the edit graph.  Normally
/// a snake has a start end end point (and it is possible for a snake to have
/// a length of zero, meaning the start and end points are the same) however
/// we do not need the end point which is why it's not implemented here.
///
/// The divide part of a divide-and-conquer strategy. A D-path has D+1 snakes
/// some of which may be empty. The divide step requires finding the ceil(D/2) +
/// 1 or middle snake of an optimal D-path. The idea for doing so is to
/// simultaneously run the basic algorithm in both the forward and reverse
/// directions until furthest reaching forward and reverse paths starting at
/// opposing corners 'overlap'.
fn find_middle_snake<T>(
    old: &[Token<T>],
    old_range: Range<usize>,
    new: &[Token<T>],
    new_range: Range<usize>,
    forward_endpoints: &mut FurthestEndpoints,
    backward_endpoints: &mut FurthestEndpoints,
) -> Option<(usize, usize)>
where
    T: PartialEq + Clone + Debug,
{
    let old_len = old_range.len();
    let new_len = new_range.len();

    let old_len_signed = isize::try_from(old_len).expect("old_len must fit in isize");
    let new_len_signed = isize::try_from(new_len).expect("new_len must fit in isize");

    // By Lemma 1 in the paper, the optimal edit script length is odd or even as
    // `delta` is odd or even.
    let delta = old_len_signed - new_len_signed;
    let delta_is_odd = delta & 1 == 1;

    // The initial point at (0, -1)
    forward_endpoints[1] = 0;
    // The initial point at (N, M+1)
    backward_endpoints[1] = 0;

    let max_edit_distance = (old_len + new_len).div_ceil(2) + 1;
    assert!(forward_endpoints.len() >= max_edit_distance);
    assert!(backward_endpoints.len() >= max_edit_distance);

    let max_edit_distance_signed =
        isize::try_from(max_edit_distance).expect("max_edit_distance must fit in isize");

    for edit_distance in 0..max_edit_distance_signed {
        // Tighter diagonal bounds: on diagonal k = x - y the constraints
        // 0 <= x <= old_len and 0 <= y <= new_len give k in [-new_len, old_len].
        // Intersect with the algorithm's [-edit_distance, edit_distance]
        // range and snap to the correct parity (k advances in steps of 2).
        let forward_diagonal_lo =
            align_lower_bound((-edit_distance).max(-new_len_signed), edit_distance);
        let forward_diagonal_hi =
            align_upper_bound(edit_distance.min(old_len_signed), edit_distance);

        // Forward path
        for diagonal in (forward_diagonal_lo..=forward_diagonal_hi).rev().step_by(2) {
            let mut old_idx = if diagonal == -edit_distance
                || (diagonal != edit_distance
                    && forward_endpoints[diagonal - 1] < forward_endpoints[diagonal + 1])
            {
                forward_endpoints[diagonal + 1]
            } else {
                forward_endpoints[diagonal - 1] + 1
            };
            let new_idx = usize::try_from(
                isize::try_from(old_idx).expect("old_idx must fit in isize") - diagonal,
            )
            .expect("old_idx - diagonal must be non-negative and fit in usize");

            // The coordinate of the start of a snake
            let (snake_start_old, snake_start_new) = (old_idx, new_idx);

            // While these sequences are identical, keep moving through the
            // graph with no cost
            if old_idx < old_range.len() && new_idx < new_range.len() {
                let advance = common_prefix_len(
                    old,
                    old_range.start + old_idx..old_range.end,
                    new,
                    new_range.start + new_idx..new_range.end,
                );
                old_idx += advance;
            }

            // This is the new best x value
            forward_endpoints[diagonal] = old_idx;

            // Only check for connections from the forward search when N - M is
            // odd and when there is a reciprocal k line coming from the other
            // direction. Forward diagonal k maps to backward diagonal
            // (delta - k). Overlap occurs when the combined forward + backward
            // reach covers the full width:
            //   forward_endpoints[k] + backward_endpoints[delta - k] >= old_len.
            if delta_is_odd
                && (diagonal - delta).abs() <= (edit_distance - 1)
                && forward_endpoints[diagonal] + backward_endpoints[-(diagonal - delta)] >= old_len
            {
                return Some((
                    snake_start_old + old_range.start,
                    snake_start_new + new_range.start,
                ));
            }
        }

        let backward_diagonal_lo =
            align_lower_bound((-edit_distance).max(-new_len_signed), edit_distance);
        let backward_diagonal_hi =
            align_upper_bound(edit_distance.min(old_len_signed), edit_distance);

        // Backward path
        for diagonal in (backward_diagonal_lo..=backward_diagonal_hi)
            .rev()
            .step_by(2)
        {
            let mut old_idx = if diagonal == -edit_distance
                || (diagonal != edit_distance
                    && backward_endpoints[diagonal - 1] < backward_endpoints[diagonal + 1])
            {
                backward_endpoints[diagonal + 1]
            } else {
                backward_endpoints[diagonal - 1] + 1
            };
            let mut new_idx = usize::try_from(
                isize::try_from(old_idx).expect("old_idx must fit in isize") - diagonal,
            )
            .expect("old_idx - diagonal must be non-negative and fit in usize");

            // Extend the snake backward (matching suffix)
            if old_idx < old_len && new_idx < new_len {
                let advance = common_suffix_len(
                    old,
                    old_range.start..old_range.start + old_len - old_idx,
                    new,
                    new_range.start..new_range.start + new_len - new_idx,
                );
                old_idx += advance;
                new_idx += advance;
            }

            // This is the new best x value
            backward_endpoints[diagonal] = old_idx;

            if !delta_is_odd
                && (diagonal - delta).abs() <= edit_distance
                && backward_endpoints[diagonal] + forward_endpoints[-(diagonal - delta)] >= old_len
            {
                return Some((
                    old_len - old_idx + old_range.start,
                    new_len - new_idx + new_range.start,
                ));
            }
        }
    }

    None
}

fn conquer<T>(
    old: &[Token<T>],
    mut old_range: Range<usize>,
    new: &[Token<T>],
    mut new_range: Range<usize>,
    forward_endpoints: &mut FurthestEndpoints,
    backward_endpoints: &mut FurthestEndpoints,
    result: &mut Vec<RawOperation<T>>,
) where
    T: PartialEq + Clone + Debug,
{
    // Check for common prefix
    let prefix_len = common_prefix_len(old, old_range.clone(), new, new_range.clone());
    if prefix_len > 0 {
        result.extend(
            old[old_range.start..old_range.start + prefix_len]
                .iter()
                .map(|token| RawOperation::Equal(vec![token.clone()])),
        );
    }
    old_range.start += prefix_len;
    new_range.start += prefix_len;

    // Check for common suffix
    let suffix_len = common_suffix_len(old, old_range.clone(), new, new_range.clone());
    let suffix_start = old_range.end - suffix_len;
    old_range.end -= suffix_len;
    new_range.end -= suffix_len;

    if old_range.is_empty() && new_range.is_empty() {
        // do nothing
    } else if new_range.is_empty() {
        result.extend(
            old[old_range.start..old_range.end]
                .iter()
                .map(|token| RawOperation::Delete(vec![token.clone()])),
        );
    } else if old_range.is_empty() {
        result.extend(
            new[new_range.start..new_range.end]
                .iter()
                .map(|token| RawOperation::Insert(vec![token.clone()])),
        );
    } else if let Some((split_old, split_new)) = find_middle_snake(
        old,
        old_range.clone(),
        new,
        new_range.clone(),
        forward_endpoints,
        backward_endpoints,
    ) {
        let (old_before, old_after) = split_at(old_range, split_old);
        let (new_before, new_after) = split_at(new_range, split_new);
        conquer(
            old,
            old_before,
            new,
            new_before,
            forward_endpoints,
            backward_endpoints,
            result,
        );
        conquer(
            old,
            old_after,
            new,
            new_after,
            forward_endpoints,
            backward_endpoints,
            result,
        );
    } else {
        result.extend(
            old[old_range.start..old_range.end]
                .iter()
                .map(|token| RawOperation::Delete(vec![token.clone()])),
        );
        result.extend(
            new[new_range.start..new_range.end]
                .iter()
                .map(|token| RawOperation::Insert(vec![token.clone()])),
        );
    }

    if suffix_len > 0 {
        result.extend(
            old[suffix_start..suffix_start + suffix_len]
                .iter()
                .map(|token| RawOperation::Equal(vec![token.clone()])),
        );
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn test_empty_diff() {
        let old: Vec<Token<String>> = vec![];
        let new: Vec<Token<String>> = vec![];
        let result = myers_diff(&old, &new);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_identical_content() {
        let content = vec!["a".into(), "b".into(), "c".into()];
        let result = myers_diff(&content, &content);
        assert_debug_snapshot!(result);
    }

    #[test]
    fn test_insert_only() {
        let old: Vec<Token<String>> = vec![];
        let new: Vec<Token<String>> = vec!["a".into(), "b".into()];
        let result = myers_diff(&old, &new);
        assert_debug_snapshot!(result);
    }

    #[test]
    fn test_delete_only() {
        let old = vec!["a".into(), "b".into()];
        let new: Vec<Token<String>> = vec![];
        let result = myers_diff(&old, &new);
        assert_debug_snapshot!(result);
    }

    #[test]
    fn test_prefix_and_suffix() {
        let old = vec!["a".into(), "b".into(), "c".into(), "d".into()];
        let new = vec!["a".into(), "x".into(), "d".into()];
        let result = myers_diff(&old, &new);
        assert_debug_snapshot!(result);
    }

    #[test]
    fn test_complex_diff() {
        let old = vec!["a".into(), "b".into(), "c".into(), "d".into()];
        let new = vec!["a".into(), "x".into(), "c".into(), "y".into()];
        let result = myers_diff(&old, &new);
        assert_debug_snapshot!(result);
    }
}
