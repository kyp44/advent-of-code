use std::str::FromStr;

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
        vec![Unsigned(1588178), Unsigned(3783758)],
        "2x3x4",
    vec![58u64, 34].answer_vec(),
    "1x1x10",
    vec![43u64, 14].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use itertools::Itertools;
    use nom::{bytes::complete::tag, multi::separated_list1, Finish};

    /// A present with specific dimensions.
    struct Present {
        /// List of dimensions in feet.
        /// This will always be sorted from least to greatest
        dimensions: Vec<u64>,
    }
    impl FromStr for Present {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut dimensions = separated_list1::<_, _, _, NomParseError, _, _>(
                tag("x"),
                nom::character::complete::u64,
            )(s)
            .finish()
            .discard_input()?;

            if dimensions.len() != 3 {
                return Err(AocError::InvalidInput(
                    format!(
                        "Present {} has {} dimensions when 3 are expected",
                        s,
                        dimensions.len()
                    )
                    .into(),
                ));
            }
            dimensions.sort_unstable();

            Ok(Present { dimensions })
        }
    }
    impl Present {
        /// Calculates the paper required to wrap this present in square feet.
        fn needed_paper(&self) -> u64 {
            self.dimensions
                .iter()
                .combinations(2)
                .map(|cv| 2 * cv.into_iter().product::<u64>())
                .sum::<u64>()
                + self.dimensions[0] * self.dimensions[1]
        }

        /// Calculates the ribbon required to wrap this present in feet.
        fn needed_ribbon(&self) -> u64 {
            self.dimensions[..2].iter().map(|d| 2 * *d).sum::<u64>()
                + self.dimensions.iter().product::<u64>()
        }
    }

    /// A list of presents that can be parsed from text input.
    pub struct Presents {
        /// The list of presents.
        presents: Vec<Present>,
    }
    impl FromStr for Presents {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Presents {
                presents: s
                    .lines()
                    .map(|line| line.parse())
                    .collect::<Result<_, _>>()?,
            })
        }
    }
    impl Presents {
        /// Calculates the total wrapping paper needed to wrap all the presents in square feet.
        pub fn needed_paper(&self) -> u64 {
            self.presents.iter().map(|p| p.needed_paper()).sum()
        }

        /// Calculates the total ribbon paper needed to wrap all the presents in feet.
        pub fn needed_ribbon(&self) -> u64 {
            self.presents.iter().map(|p| p.needed_ribbon()).sum()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 2,
    name: "I Was Told There Would Be No Math",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let presents: Presents = input.expect_input()?.parse()?;

            // Process
            Ok(presents.needed_paper().into())
        },
        // Part b)
        |input| {
            // Generation
            let presents: Presents = input.expect_input()?.parse()?;

            // Process
            Ok(presents.needed_ribbon().into())
        },
    ],
};
