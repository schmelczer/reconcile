//! Taken from https://github.com/mitsuhiko/similar/blob/7e15c44de11a1cd61e1149189929e189ef977fd8/src/algorithms/myers.rs
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
    ops::{Index, IndexMut, Range},
    vec,
};

use super::raw_operation::RawOperation;
use crate::{
    tokenizer::token::Token,
    utils::{common_prefix_len::common_prefix_len, common_suffix_len::common_suffix_len},
};

/// Myers' diff algorithm with deadline.
///
/// Diff `old`, between indices `old_range` and `new` between indices
/// `new_range`.
///
/// This diff is done with an optional deadline that defines the maximal
/// execution time permitted before it bails and falls back to an approximation.
pub fn diff<T>(old: &[Token<T>], new: &[Token<T>]) -> Vec<RawOperation<T>>
where
    T: PartialEq + Clone,
{
    let max_d = max_d(old.len(), new.len());
    let mut vb = V::new(max_d);
    let mut vf = V::new(max_d);
    let mut result: Vec<RawOperation<T>> = vec![];
    conquer(
        old,
        0..old.len(),
        new,
        0..new.len(),
        &mut vf,
        &mut vb,
        &mut result,
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
    v: Vec<usize>, // Look into initializing this to -1 and storing isize
}

impl V {
    fn new(max_d: usize) -> Self {
        Self {
            offset: max_d as isize,
            v: vec![0; 2 * max_d],
        }
    }

    fn len(&self) -> usize { self.v.len() }
}

impl Index<isize> for V {
    type Output = usize;

    fn index(&self, index: isize) -> &Self::Output { &self.v[(index + self.offset) as usize] }
}

impl IndexMut<isize> for V {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        &mut self.v[(index + self.offset) as usize]
    }
}

fn max_d(len1: usize, len2: usize) -> usize {
    // XXX look into reducing the need to have the additional '+ 1'
    (len1 + len2 + 1) / 2 + 1
}

#[inline(always)]
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
    T: PartialEq + Clone,
{
    let n = old_range.len();
    let m = new_range.len();

    // By Lemma 1 in the paper, the optimal edit script length is odd or even as
    // `delta` is odd or even.
    let delta = n as isize - m as isize;
    let odd = delta & 1 == 1;

    // The initial point at (0, -1)
    vf[1] = 0;
    // The initial point at (N, M+1)
    vb[1] = 0;

    // We only need to explore ceil(D/2) + 1
    let d_max = max_d(n, m);
    assert!(vf.len() >= d_max);
    assert!(vb.len() >= d_max);

    for d in 0..d_max as isize {
        // Forward path
        for k in (-d..=d).rev().step_by(2) {
            let mut x = if k == -d || (k != d && vf[k - 1] < vf[k + 1]) {
                vf[k + 1]
            } else {
                vf[k - 1] + 1
            };
            let y = (x as isize - k) as usize;

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
            let mut y = (x as isize - k) as usize;

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
    T: PartialEq + Clone,
{
    // Check for common prefix
    let common_prefix_len = common_prefix_len(old, old_range.clone(), new, new_range.clone());
    if common_prefix_len > 0 {
        result.push(RawOperation::Equal(
            old[old_range.start..old_range.start + common_prefix_len].to_vec(),
        ));
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
        // Do nothing
    } else if new_range.is_empty() {
        result.push(RawOperation::Delete(
            old[old_range.start..old_range.start + old_range.len()].to_vec(),
        ));
    } else if old_range.is_empty() {
        result.push(RawOperation::Insert(
            new[new_range.start..new_range.start + new_range.len()].to_vec(),
        ));
    } else if let Some((x_start, y_start)) =
        find_middle_snake(old, old_range.clone(), new, new_range.clone(), vf, vb)
    {
        let (old_a, old_b) = split_at(old_range, x_start);
        let (new_a, new_b) = split_at(new_range, y_start);
        conquer(old, old_a, new, new_a, vf, vb, result);
        conquer(old, old_b, new, new_b, vf, vb, result);
    } else {
        result.push(RawOperation::Delete(
            old[old_range.start..old_range.end].to_vec(),
        ));
        result.push(RawOperation::Insert(
            new[new_range.start..new_range.end].to_vec(),
        ));
    }

    if common_suffix_len > 0 {
        result.push(RawOperation::Equal(
            old[common_suffix.0..common_suffix.0 + common_suffix_len].to_vec(),
        ));
    }
}
