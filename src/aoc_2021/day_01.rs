use aoc::prelude::*;
use itertools::Itertools;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(1696), Unsigned(1737)],
    "199
200
208
210
200
207
240
269
260
263",
    vec![7u64, 5].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;

    /// Extension for [`Iterator`]s.
    pub trait CountIncreases {
        /// Counts the number of times that the values yielded by the [`Iterator`] increase.
        fn count_increases(self) -> u64;
    }
    impl<I: Iterator> CountIncreases for I
    where
        <I as Iterator>::Item: Clone + PartialOrd,
    {
        fn count_increases(self) -> u64 {
            self.tuple_windows().filter_count(|(a, b)| a < b)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 1,
    name: "Sonar Sweep",
    preprocessor: Some(|input| Ok(Box::new(u64::gather(input.lines())?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Vec<u64>>()?
                .iter()
                .count_increases()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Vec<u64>>()?
                .iter()
                .tuple_windows()
                .map(|(a, b, c)| a + b + c)
                .count_increases()
                .into())
        },
    ],
};
