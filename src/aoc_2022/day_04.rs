use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
    example {
    input = "2-4,6-8
2-3,4-5
5-7,7-9
2-8,3-7
6-6,4-6
2-6,4-8";
    answers = vec![2u64].answer_vec();
    }
    actual_answers = vec![Unsigned(123)];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 4,
    name: "Camp Cleanup",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation

            // Process
            Ok(0u64.into())
        },
    ],
};
