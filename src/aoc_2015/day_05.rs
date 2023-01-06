use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(236), Unsigned(51)],
    "ugknbfddgicrmopn
aaa
jchzalrnumimnmhp
haegwjzuvuyypxyu
dvszwmarrgswjxmb",
    vec![Some(Unsigned(2)), None],
    "qjhvhtzxzqqjkmpb
xxyxx
uurcxstgmygtbstg
ieodomkazucvgmuy",
    vec![None, Some(Unsigned(2))]
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use itertools::Itertools;

    /// Simply denotes which part of the problem we are solving.
    pub trait Part {}
    /// Denotes part a).
    pub struct PartA;
    impl Part for PartA {}
    /// Denotes part b).
    pub struct PartB;
    impl Part for PartB {}

    /// Provides a method to identify if something is nice.
    pub trait Nice<P: Part> {
        /// Determines whether the object is nice (`true`) or naughty (`false`).
        fn is_nice(&self) -> bool;
    }
    impl Nice<PartA> for &str {
        fn is_nice(&self) -> bool {
            /// Array of English vowels, not including y.
            const VOWELS: &[char] = &['a', 'e', 'i', 'o', 'u'];
            /// Array of forbidden strings.
            const BAD_STRS: &[&str] = &["ab", "cd", "pq", "xy"];

            // Check vowels
            FilterCount::<_, usize>::filter_count(self.chars(), |c| VOWELS.contains(c)) >= 3
                // Double letters
                    && self
                        .chars()
                        .tuple_windows()
                    .any(|(a, b)| a.is_alphabetic() && a == b)
                // Does not contain any forbidden strings
                && BAD_STRS.iter().all(|bs| !self.contains(bs))
        }
    }
    impl Nice<PartB> for &str {
        fn is_nice(&self) -> bool {
            // Pair of letters appearing twice but not overlapping
            (0..self.len() - 3).any(|i| self[i + 2..].contains(&self[i..i + 2]))
	    &&
        // Repeating letter with one in between
        self.chars().tuple_windows().any(|(a, _, c)| a == c)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 5,
    name: "Doesn't He Have Intern-Elves For This?",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_input()?
                    .lines()
                    .filter_count(|s| Nice::<PartA>::is_nice(s)),
            ))
        },
        // Part b)
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_input()?
                    .lines()
                    .filter_count(|s| Nice::<PartB>::is_nice(s)),
            ))
        },
    ],
};
