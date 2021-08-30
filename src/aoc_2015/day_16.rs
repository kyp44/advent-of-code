use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(0)],
    "",
    vec![0u64].answer_vec()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 16,
    name: "Aunt Sue",
    solvers: &[
        // Part a)
        |input| {
            // Generation

            // Process
            Ok(0u64.into())
        },
    ],
};
