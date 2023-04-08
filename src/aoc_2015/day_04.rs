use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "abcdef";
            answers = vec![609043u64].answer_vec();
        }
        example {
            input = "pqrstuv";
            answers = vec![1048970u64].answer_vec();
        }
        actual_answers = vec![Unsigned(254575), Unsigned(1038736)];
    }
}

/// Contains solution implementation items.
mod solution {
    /// Behavior specific to each part of the problem.
    pub trait Part {
        /// Tests whether the third byte of a MD5 hash meets the criteria for this part.
        fn check_third_byte(byte: u8) -> bool;
    }

    /// Behavior for part one.
    pub struct PartOne;
    impl Part for PartOne {
        fn check_third_byte(byte: u8) -> bool {
            byte & 0xF0 == 0
        }
    }
    /// Behavior for part two.
    pub struct PartTwo;
    impl Part for PartTwo {
        fn check_third_byte(byte: u8) -> bool {
            byte == 0
        }
    }

    /// Solves either part for the given text key.
    ///
    /// Returns the lowest positive integer such that the MD5 hash begins with the
    /// appropriate number of zeros when appended to the key.
    pub fn solve<P: Part>(input: &str) -> u64 {
        let input = input.trim();

        let mut ans: u64 = 0;
        loop {
            let hash = md5::compute(format!("{input}{ans}"));

            // Check that the first hex digits are zero
            if hash[0] == 0 && hash[1] == 0 && P::check_third_byte(hash[2]) {
                break ans;
            }
            ans += 1;
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 4,
    name: "The Ideal Stocking Stuffer",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| Ok(solve::<PartOne>(input.expect_input()?).into()),
        // Part two
        |input| Ok(solve::<PartTwo>(input.expect_input()?).into()),
    ],
};
