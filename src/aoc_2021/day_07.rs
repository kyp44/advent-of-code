use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "16,1,2,0,4,2,7,1,2,14";
            answers = unsigned![37, 168];
        }
        actual_answers = unsigned![341534, 93397632];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use itertools::{Itertools, MinMaxResult};

    /// Behavior specific to one particular part of the problem.
    pub trait Part {
        /// Returns the amount of fuel used for a given horizontal distance traveled.
        fn fuel_used(dist: u64) -> u64;
    }

    /// Behavior for part one.
    pub struct PartOne {}
    impl Part for PartOne {
        fn fuel_used(dist: u64) -> u64 {
            dist
        }
    }

    /// Behavior for part two.
    pub struct PartTwo {}
    impl Part for PartTwo {
        fn fuel_used(dist: u64) -> u64 {
            dist * (dist + 1) / 2
        }
    }

    /// Collection of crab submarines, which can be parsed from text input.
    pub struct CrabSubs {
        /// The horizontal positions of each crab.
        positions: Box<[u64]>,
    }
    impl FromStr for CrabSubs {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(CrabSubs {
                positions: u64::from_csv(s)?.into_boxed_slice(),
            })
        }
    }
    impl CrabSubs {
        /// Determines the optimum way for the crabs to all align to the same
        /// horizontal position, and returns the amount of fuel needed for this
        /// based on the fuel usage requirements for the [`Part`].
        pub fn align<P: Part>(&self) -> AocResult<u64> {
            match self.positions.iter().minmax() {
                MinMaxResult::MinMax(min, max) => Ok(((*min)..=(*max))
                    .map(|p| {
                        self.positions
                            .iter()
                            .map(|x| P::fuel_used(x.abs_diff(p)))
                            .sum()
                    })
                    .min()
                    .unwrap()),
                MinMaxResult::OneElement(v) => Ok(*v),
                _ => Err(AocError::Process("Data empty!".into())),
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 7,
    name: "The Treachery of Whales",
    preprocessor: Some(|input| Ok(Box::new(CrabSubs::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<CrabSubs>()?.align::<PartOne>()?.into())
        },
        // Part two
        |input| {
            // Process
            Ok(input.expect_data::<CrabSubs>()?.align::<PartTwo>()?.into())
        },
    ],
};
