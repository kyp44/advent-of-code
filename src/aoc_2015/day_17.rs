use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "20
15
10
5
5";
            answers = unsigned![0, 0];
        }
        actual_answers = unsigned![654, 57];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use itertools::Itertools;

    /// Definition of the problem that can be parsed from text input.
    pub struct Problem {
        /// The containers that we have in liters.
        containers: Box<[u16]>,
    }
    impl FromStr for Problem {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Problem {
                containers: u16::gather(s.lines())?.into_boxed_slice(),
            })
        }
    }
    impl Problem {
        /// Returns an [`Iterator`] of all combinations of containers that hold
        /// a particular amount of eggnog in liters.
        pub fn combinations(&self, amount: u16) -> impl Iterator<Item = Vec<u16>> + '_ {
            (1..=self.containers.len())
                .flat_map(move |k| {
                    self.containers
                        .iter()
                        .combinations(k)
                        .map(|c| c.into_iter().copied().collect())
                })
                .filter(move |c: &Vec<u16>| c.iter().sum::<u16>() == amount)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 17,
    name: "No Such Thing as Too Much",
    preprocessor: Some(|input| Ok(Box::new(input.parse::<Problem>()?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<Problem>()?
                    .combinations(150)
                    .count()
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            // Process
            let combs: Vec<Vec<u16>> = input.expect_data::<Problem>()?.combinations(150).collect();
            let min = combs.iter().map(|cv| cv.len()).min().unwrap_or(0);
            let ans: u64 = combs.iter().filter_count(|cv| cv.len() == min);

            Ok(ans.into())
        },
    ],
};
