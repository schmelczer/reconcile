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
//! # Heuristics
//!
//! At present this implementation of Myers' does not implement any more
//! advanced heuristics that would solve some pathological cases.  For instance
//! passing two large and completely distinct sequences to the algorithm will
//! make it spin without making reasonable progress.
//! For potential improvements here see [similar#15](https://github.com/mitsuhiko/similar/issues/15).

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
    let max_d = (old.len() + new.len()).div_ceil(2) + 1;
    let mut vb = V::new(max_d);
    let mut vf = V::new(max_d);
    let mut result = Vec::new();

    conquer(
        old,
        0..old.len(),
        new,
        0..new.len(),
        &mut vf,
        &mut vb,
        &mut result,
    );

    debug_assert!(
        result.iter().all(|op| op.tokens().len() == 1),
        "All operations must be of length 1"
    );

    result
}

// A D-path is a path which starts at (0,0) that has exactly D non-diagonal
// edges. All D-paths consist of a (D - 1)-path followed by a non-diagonal edge
// and then a possibly empty sequence of diagonal edges called a snake.

/// `V` contains the endpoints of the furthest reaching `D-paths`. For each
/// recorded endpoint `(x,y)` in diagonal `k`, we only need to retain `x`
/// because `y` can be computed from `x - k`. In other words, `V` is an array of
/// integers where `V[k]` contains the row index of the endpoint of the furthest
/// reaching path in diagonal `k`.
///
/// We can't use a traditional Vec to represent `V` since we use `k` as an index
/// and it can take on negative values. So instead `V` is represented as a
/// light-weight wrapper around a Vec plus an `offset` which is the maximum
/// value `k` can take on in order to map negative `k`'s back to a value >= 0.
#[derive(Debug)]
struct V {
    offset: isize,
    v: Vec<usize>,
}

impl V {
    fn new(max_d: usize) -> Self {
        // max_d should fit in isize for the algorithm to work correctly
        let offset = isize::try_from(max_d).unwrap_or(isize::MAX);
        Self {
            offset,
            v: vec![0; 2 * max_d],
        }
    }

    fn len(&self) -> usize { self.v.len() }
}

impl Index<isize> for V {
    type Output = usize;

    fn index(&self, index: isize) -> &Self::Output {
        let idx = usize::try_from(index + self.offset).unwrap_or(usize::MAX);
        &self.v[idx.min(self.v.len().saturating_sub(1))]
    }
}

impl IndexMut<isize> for V {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        let idx = usize::try_from(index + self.offset).unwrap_or(usize::MAX);
        let len = self.v.len();
        &mut self.v[idx.min(len.saturating_sub(1))]
    }
}

fn split_at(range: Range<usize>, at: usize) -> (Range<usize>, Range<usize>) {
    (range.start..at, at..range.end)
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
    vf: &mut V,
    vb: &mut V,
) -> Option<(usize, usize)>
where
    T: PartialEq + Clone + Debug,
{
    let n = old_range.len();
    let m = new_range.len();

    // By Lemma 1 in the paper, the optimal edit script length is odd or even as
    // `delta` is odd or even.
    let delta = isize::try_from(n).unwrap_or(isize::MAX) - isize::try_from(m).unwrap_or(isize::MAX);
    let odd = delta & 1 == 1;

    // The initial point at (0, -1)
    vf[1] = 0;
    // The initial point at (N, M+1)
    vb[1] = 0;

    let d_max = (n + m).div_ceil(2) + 1;
    assert!(vf.len() >= d_max);
    assert!(vb.len() >= d_max);

    let d_max_isize = isize::try_from(d_max).unwrap_or(isize::MAX);
    for d in 0..d_max_isize {
        // Forward path
        for k in (-d..=d).rev().step_by(2) {
            let mut x = if k == -d || (k != d && vf[k - 1] < vf[k + 1]) {
                vf[k + 1]
            } else {
                vf[k - 1] + 1
            };
            let y = usize::try_from(isize::try_from(x).unwrap_or(isize::MAX) - k).unwrap_or(0);

            // The coordinate of the start of a snake
            let (x0, y0) = (x, y);
            //  While these sequences are identical, keep moving through the
            //  graph with no cost
            if x < old_range.len() && y < new_range.len() {
                let advance = common_prefix_len(
                    old,
                    old_range.start + x..old_range.end,
                    new,
                    new_range.start + y..new_range.end,
                );
                x += advance;
            }

            // This is the new best x value
            vf[k] = x;

            // Only check for connections from the forward search when N - M is
            // odd and when there is a reciprocal k line coming from the other
            // direction.
            if odd && (k - delta).abs() <= (d - 1) {
                // TODO optimize this so we don't have to compare against n
                if vf[k] + vb[-(k - delta)] >= n {
                    // Return the snake
                    return Some((x0 + old_range.start, y0 + new_range.start));
                }
            }
        }

        // Backward path
        for k in (-d..=d).rev().step_by(2) {
            let mut x = if k == -d || (k != d && vb[k - 1] < vb[k + 1]) {
                vb[k + 1]
            } else {
                vb[k - 1] + 1
            };
            let mut y = usize::try_from(isize::try_from(x).unwrap_or(isize::MAX) - k).unwrap_or(0);

            // The coordinate of the start of a snake
            if x < n && y < m {
                let advance = common_suffix_len(
                    old,
                    old_range.start..old_range.start + n - x,
                    new,
                    new_range.start..new_range.start + m - y,
                );
                x += advance;
                y += advance;
            }

            // This is the new best x value
            vb[k] = x;

            if !odd && (k - delta).abs() <= d {
                // TODO optimize this so we don't have to compare against n
                if vb[k] + vf[-(k - delta)] >= n {
                    // Return the snake
                    return Some((n - x + old_range.start, m - y + new_range.start));
                }
            }
        }

        // TODO: Maybe there's an opportunity to optimize and bail early?
    }

    None
}

fn conquer<T>(
    old: &[Token<T>],
    mut old_range: Range<usize>,
    new: &[Token<T>],
    mut new_range: Range<usize>,
    vf: &mut V,
    vb: &mut V,
    result: &mut Vec<RawOperation<T>>,
) where
    T: PartialEq + Clone + Debug,
{
    // Check for common prefix
    let common_prefix_len = common_prefix_len(old, old_range.clone(), new, new_range.clone());
    if common_prefix_len > 0 {
        result.extend(
            old[old_range.start..old_range.start + common_prefix_len]
                .iter()
                .map(|token| RawOperation::Equal(vec![token.clone()])),
        );
    }
    old_range.start += common_prefix_len;
    new_range.start += common_prefix_len;

    // Check for common suffix
    let common_suffix_len = common_suffix_len(old, old_range.clone(), new, new_range.clone());
    let common_suffix = (
        old_range.end - common_suffix_len,
        new_range.end - common_suffix_len,
    );
    old_range.end -= common_suffix_len;
    new_range.end -= common_suffix_len;

    if old_range.is_empty() && new_range.is_empty() {
        // do nothing
    } else if new_range.is_empty() {
        result.extend(
            old[old_range.start..old_range.start + old_range.len()]
                .iter()
                .map(|token| RawOperation::Delete(vec![token.clone()])),
        );
    } else if old_range.is_empty() {
        result.extend(
            new[new_range.start..new_range.start + new_range.len()]
                .iter()
                .map(|token| RawOperation::Insert(vec![token.clone()])),
        );
    } else if let Some((x_start, y_start)) =
        find_middle_snake(old, old_range.clone(), new, new_range.clone(), vf, vb)
    {
        let (old_a, old_b) = split_at(old_range, x_start);
        let (new_a, new_b) = split_at(new_range, y_start);
        conquer(old, old_a, new, new_a, vf, vb, result);
        conquer(old, old_b, new, new_b, vf, vb, result);
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

    if common_suffix_len > 0 {
        result.extend(
            old[common_suffix.0..common_suffix.0 + common_suffix_len]
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
