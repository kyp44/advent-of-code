//! Collection of extension methods for various items that involve iteration.
//!
//! This includes the [`IteratorExt`] trait, which provides iterator adapter methods,
//! and the [`StrExt`] trait, which provides methods to iterate over strings.

use itertools::{Itertools, MinMaxResult};
use std::{fmt::Debug, ops::RangeInclusive};

/// Extension methods for [`Iterator`]s.
pub trait IteratorExt<T> {
    /// This is a convenience method to count the elements of an iterator after filtering by
    /// some predicate.
    ///
    /// The numeric return type is anything that can be fallibly
    /// converted from a [`usize`]. An empty iterator will of course return zero
    /// regardless of the predicate
    ///
    /// # Panics
    ///
    /// Panics if the [`usize`] count cannot be converted into the numeric return type.
    ///
    /// # Example
    ///
    /// ```
    /// use aoc::prelude::*;
    ///
    /// assert_eq!(std::iter::empty::<u8>().filter_count::<usize>(|_| true), 0);
    /// assert_eq!([-1, 3, 5, -7, 0, 8, -9, -2, 5].into_iter().filter_count::<u32>(|x| *x <= 0), 5);
    /// ```
    // TODO: Triggering a lint because of the code at the end.
    fn filter_count<O: TryFrom<usize>>(self, f: impl Fn(&T) -> bool) -> O
    where
        <O as TryFrom<usize>>::Error: Debug;

    /// Returns an inclusive range for an [`Iterator`] over applicable ordered types.
    ///
    /// Will return `None` if the iterator is empty, and the single-element range
    /// `x..=x` if the iterator yields only a single element `x`.
    ///
    /// # Example
    ///
    /// ```
    /// use aoc::prelude::*;
    ///
    /// assert_eq!(std::iter::empty::<u8>().range(), None);
    /// assert_eq!([5u8].into_iter().range(), Some(5..=5));
    /// assert_eq!([-9, 4, 7, -11, 8, 5, -6, -3, 15].into_iter().range(), Some(-11..=15));
    /// ```
    fn range(self) -> Option<RangeInclusive<T>>
    where
        T: PartialOrd + Copy;

    // TODO: This should trigger the function verb lint.
    /// Advance the [`Iterator`] by some number of iterations and return the resulting item.
    ///
    /// Note that this is the same as [`Iterator::nth`], but just offset by one, which can
    /// be more convenient in some situations. If `0` is passed then `None` will be returned
    /// and likewise if the iterator is exhausted before `n` iterations.
    ///
    /// # Example
    ///
    /// ```
    /// use aoc::prelude::*;
    ///
    /// assert_eq!([0, 1, 2, 3, 4, 5, 6].into_iter().iterations(0), None);
    /// assert_eq!([0, 1, 2, 3, 4, 5, 6].into_iter().iterations(20), None);
    /// assert_eq!([0, 1, 2, 3, 4, 5, 6].into_iter().iterations(4), Some(3));
    /// ```
    fn iterations(&mut self, n: usize) -> Option<T>;
}
impl<T, I: Iterator<Item = T>> IteratorExt<T> for I {
    fn filter_count<O: TryFrom<usize>>(self, f: impl Fn(&T) -> bool) -> O
    where
        <O as TryFrom<usize>>::Error: Debug,
    {
        self.filter(f).count().try_into().unwrap()
    }

    fn range(self) -> Option<RangeInclusive<T>>
    where
        T: PartialOrd + Copy,
    {
        match self.minmax() {
            MinMaxResult::NoElements => None,
            MinMaxResult::OneElement(n) => Some(n..=n),
            MinMaxResult::MinMax(a, b) => Some(a..=b),
        }
    }

    fn iterations(&mut self, n: usize) -> Option<T> {
        if n > 0 {
            self.nth(n - 1)
        } else {
            None
        }
    }
}

/// Extension methods for iteration over strings.
pub trait StrExt {
    /// Returns an [`Iterator`] the performs substring replacements on a string, one replacement
    /// at a time, yielding the resulting string after each replacement.
    ///
    /// The replacements are
    /// independent and not cumulative.
    ///
    /// # Example
    ///
    /// ```
    /// use aoc::prelude::*;
    ///
    /// let mut replacements = "The red fox jumps over the blue fox and lands on the yellow fox".individual_replacements("fox", "dog");
    ///
    /// assert_eq!(replacements.next().unwrap(), "The red dog jumps over the blue fox and lands on the yellow fox");
    /// assert_eq!(replacements.next().unwrap(), "The red fox jumps over the blue dog and lands on the yellow fox");
    /// assert_eq!(replacements.next().unwrap(), "The red fox jumps over the blue fox and lands on the yellow dog");
    /// assert_eq!(replacements.next(), None);
    /// ```
    fn individual_replacements<'a, 'b, 'c>(
        &'a self,
        from: &'b str,
        to: &'c str,
    ) -> Replacements<'a, 'b, 'c>;

    /// Returns an [`Iterator`] over runs of repeated characters in a string.
    ///
    /// The iterator yields substrings of one or more characters that are the same.
    ///
    /// # Example
    ///
    /// ```
    /// use aoc::prelude::*;
    ///
    /// assert_eq!("".split_runs().next(), None);
    /// assert_eq!("ABCDEF".split_runs().collect::<Vec<_>>(), vec!["A", "B", "C", "D", "E", "F"]);
    /// assert_eq!("abbbcddddeefggg".split_runs().collect::<Vec<_>>(), vec!["a", "bbb", "c", "dddd", "ee", "f", "ggg"]);
    /// ```
    fn split_runs(&self) -> Runs;
}
impl StrExt for str {
    fn individual_replacements<'a, 'b, 'c>(
        &'a self,
        from: &'b str,
        to: &'c str,
    ) -> Replacements<'a, 'b, 'c> {
        Replacements {
            original: self,
            idx: 0,
            from,
            to,
        }
    }

    fn split_runs(&self) -> Runs {
        Runs { remaining: self }
    }
}

/// [`Iterator`] to perform string replacements.
///
/// See [`StrExt::individual_replacements`].
pub struct Replacements<'a, 'b, 'c> {
    /// Original string.
    original: &'a str,
    /// Current index in the string.
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

/// [`Iterator`] over runs of the same characters in strings.
///
/// See [`StrExt::split_runs`].
pub struct Runs<'a> {
    /// String portion remaining after the current replacement.
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
