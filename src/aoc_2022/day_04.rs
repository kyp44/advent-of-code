use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "2-4,6-8
2-3,4-5
5-7,7-9
2-8,3-7
6-6,4-6
2-6,4-8";
            answers = unsigned![2, 4];
        }
        actual_answers = unsigned![595, 952];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::ops::RangeInclusive;

    use aoc::parse::inclusive_range;
    use nom::{bytes::complete::tag, combinator::map, sequence::separated_pair};

    use super::*;

    /// Section assignments for a pair of elves.
    pub struct Assignment {
        /// The section range for the first elf.
        elf_1: RangeInclusive<u8>,
        /// The section range for the second elf.
        elf_2: RangeInclusive<u8>,
    }
    impl Parsable<'_> for Assignment {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(
                    inclusive_range(nom::character::complete::u8),
                    tag(","),
                    inclusive_range(nom::character::complete::u8),
                ),
                |(elf_1, elf_2)| Self { elf_1, elf_2 },
            )
            .parse(input)
        }
    }
    impl Assignment {
        /// Returns whether either of the assignments totally contains the other.
        pub fn redundant(&self) -> bool {
            self.elf_1.contains_range(&self.elf_2) || self.elf_2.contains_range(&self.elf_1)
        }

        /// Returns whether the two assignments overlap at all.
        pub fn overlaps(&self) -> bool {
            self.elf_1.intersection(&self.elf_2).is_some()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 4,
    name: "Camp Cleanup",
    preprocessor: Some(|input| Ok(Box::new(Assignment::gather(input.lines())?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Vec<Assignment>>()?
                .iter()
                .filter_count::<u64>(|a| Assignment::redundant(a))
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Vec<Assignment>>()?
                .iter()
                .filter_count::<u64>(|a| Assignment::overlaps(a))
                .into())
        },
    ],
};
