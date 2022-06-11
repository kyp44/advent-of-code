use itertools::Itertools;

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

trait Part {}
struct PartA;
impl Part for PartA {}
struct PartB;
impl Part for PartB {}

trait Nice<P: Part> {
    fn is_nice(&self) -> bool;
}
impl Nice<PartA> for &str {
    fn is_nice(&self) -> bool {
        const VOWELS: &[char] = &['a', 'e', 'i', 'o', 'u'];
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

pub const SOLUTION: Solution = Solution {
    day: 5,
    name: "Doesn't He Have Intern-Elves For This?",
    solvers: &[
        // Part a)
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_input()?
                    .lines()
                    .filter_count(|s| Nice::<PartA>::is_nice(s)),
            )
            .into())
        },
        // Part b)
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_input()?
                    .lines()
                    .filter_count(|s| Nice::<PartB>::is_nice(s)),
            )
            .into())
        },
    ],
};
