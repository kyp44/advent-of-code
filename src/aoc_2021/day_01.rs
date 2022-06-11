use crate::aoc::prelude::*;

use itertools::Itertools;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
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

trait CountIncreases {
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

pub const SOLUTION: Solution = Solution {
    day: 1,
    name: "Sonar Sweep",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let depths = u64::gather(input.expect_input()?.lines())?;

            // Process
            Ok(depths.into_iter().count_increases().into())
        },
        // Part b)
        |input| {
            // Generation
            let depths = u64::gather(input.expect_input()?.lines())?;

            // Process
            Ok(depths
                .into_iter()
                .tuple_windows()
                .map(|(a, b, c)| a + b + c)
                .count_increases()
                .into())
        },
    ],
};
