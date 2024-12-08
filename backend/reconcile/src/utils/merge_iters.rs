use std::{cmp::Ordering, iter::Peekable};

pub struct MergeAscending<L, R, F, O>
where
    L: Iterator<Item = R::Item>,
    R: Iterator,
    F: Fn(&R::Item) -> O,
    O: PartialOrd,
{
    left: Peekable<L>,
    right: Peekable<R>,
    get_key: F,
}

impl<L, R, F, O> MergeAscending<L, R, F, O>
where
    L: Iterator<Item = R::Item>,
    R: Iterator,
    F: Fn(&R::Item) -> O,
    O: PartialOrd,
{
    fn new(left: L, right: R, get_key: F) -> Self {
        MergeAscending {
            left: left.peekable(),
            right: right.peekable(),
            get_key,
        }
    }
}

impl<L, R, F, O> Iterator for MergeAscending<L, R, F, O>
where
    L: Iterator<Item = R::Item>,
    R: Iterator,
    F: Fn(&R::Item) -> O,
    O: PartialOrd,
{
    type Item = L::Item;

    fn next(&mut self) -> Option<L::Item> {
        let order = match (self.left.peek(), self.right.peek()) {
            (Some(l), Some(r)) => (self.get_key)(l).partial_cmp(&(self.get_key)(r)),
            (Some(_), None) => Some(Ordering::Less),
            (None, Some(_)) => Some(Ordering::Greater),
            (None, None) => return None,
        };

        match order {
            Some(Ordering::Less) | None => self.left.next(),
            Some(Ordering::Equal) => self.left.next(),
            Some(Ordering::Greater) => self.right.next(),
        }
    }
}

pub trait MergeSorted: Iterator {
    fn merge_sorted_by_key<R, F, O>(self, other: R, get_key: F) -> MergeAscending<Self, R, F, O>
    where
        Self: Sized,
        R: Iterator<Item = Self::Item>,
        F: Fn(&Self::Item) -> O,
        O: PartialOrd,
    {
        MergeAscending::new(self, other, get_key)
    }
}

impl<T: ?Sized> MergeSorted for T where T: Iterator {}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_merge_sorted_by_key() {
        let left = [9, 7, 5, 3, 1];
        let right = [7, 6, 5, 4, 3];

        let result: Vec<i32> = left
            .into_iter()
            .merge_sorted_by_key(right.into_iter(), |x| -1 * x)
            .collect();
        assert_eq!(result, vec![9, 7, 7, 6, 5, 5, 4, 3, 3, 1]);
    }
}
