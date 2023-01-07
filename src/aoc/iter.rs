/// Collection of Iterator adapter extensions and other extensions that return Iterators.
use std::{fmt::Debug, ops::RangeInclusive};

use num::Integer;

/// Convenience function to count from a filtered Iterator.
pub trait FilterCount<T, O> {
    fn filter_count<F: Fn(&T) -> bool>(self, f: F) -> O;
}
impl<T, I, O: TryFrom<usize>> FilterCount<T, O> for I
where
    I: Iterator<Item = T>,
    <O as TryFrom<usize>>::Error: Debug,
{
    fn filter_count<F: Fn(&T) -> bool>(self, f: F) -> O {
        self.filter(f).count().try_into().unwrap()
    }
}

/// Convenience trait to get the range from an Iterator of integers.
/// Any empty iterator will just have a range of 0..1.
pub trait HasRange<T> {
    fn range(self) -> Option<RangeInclusive<T>>;
}
impl<T, I> HasRange<T> for I
where
    T: Integer + Copy,
    I: Iterator<Item = T>,
{
    fn range(self) -> Option<RangeInclusive<T>> {
        let mut min = None;
        let mut max = None;

        for x in self {
            if min.is_none() || x < min.unwrap() {
                min = Some(x);
            }
            if max.is_none() || x > max.unwrap() {
                max = Some(x);
            }
        }

        if let (Some(min), Some(max)) = (min, max) {
            Some(min..=max)
        } else {
            None
        }
    }
}

/// Iterator adapter to an Option Iterator that includes None at the beginning.
#[derive(new, Clone)]
pub struct NoneIter<I> {
    #[new(value = "false")]
    did_none: bool,
    iter: I,
}
impl<I> Iterator for NoneIter<I>
where
    I: Iterator,
{
    type Item = Option<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.did_none {
            self.iter.next().map(Some)
        } else {
            self.did_none = true;
            Some(None)
        }
    }
}

/// Adapter extension trait
pub trait HasNoneIter {
    fn none_iter(self) -> NoneIter<Self>
    where
        Self: Sized;
}
impl<I> HasNoneIter for I
where
    I: Iterator,
{
    fn none_iter(self) -> NoneIter<Self> {
        NoneIter::new(self)
    }
}

/// [Iterator] to replace occurrences in a string one at a time.
#[derive(new)]
pub struct Replacements<'a, 'b, 'c> {
    /// Original string.
    original: &'a str,
    /// Current index in teh string.
    #[new(value = "0")]
    idx: usize,
    /// Substring to replace.
    from: &'b str,
    /// String to which to replace substrings.
    to: &'c str,
}
impl Iterator for Replacements<'_, '_, '_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.original.len() {
            let (pre, post) = self.original.split_at(self.idx);
            self.idx += 1;
            if post.starts_with(self.from) {
                return Some(format!("{}{}", pre, post.replacen(self.from, self.to, 1)));
            }
        }
        None
    }
}

/// Trait to create a Replacements [Iterator]].
pub trait IndividualReplacements<'a, 'b, 'c> {
    fn individual_replacements(&'a self, from: &'b str, to: &'c str) -> Replacements<'a, 'b, 'c>;
}
impl<'a, 'b, 'c> IndividualReplacements<'a, 'b, 'c> for str {
    fn individual_replacements(&'a self, from: &'b str, to: &'c str) -> Replacements<'a, 'b, 'c> {
        Replacements::new(self, from, to)
    }
}

/// [Iterator] over runs of the same characters in strings.
pub struct Runs<'a> {
    remaining: &'a str,
}
impl<'a> Iterator for Runs<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        let first_char = self.remaining.chars().next().unwrap();
        let end = match self.remaining.chars().position(|c| c != first_char) {
            None => self.remaining.len(),
            Some(i) => i,
        };
        let next = &self.remaining[0..end];
        self.remaining = &self.remaining[end..];
        Some(next)
    }
}

/// Trait that allows splitting by runs on the same elements.
pub trait SplitRuns {
    fn split_runs(&self) -> Runs;
}
impl SplitRuns for str {
    fn split_runs(&self) -> Runs {
        Runs { remaining: self }
    }
}
