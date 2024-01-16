use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
    example {
    input = "";
    answers = vec![123u64].answer_vec();
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
    day: 5,
    name: "Supply Stack",
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
