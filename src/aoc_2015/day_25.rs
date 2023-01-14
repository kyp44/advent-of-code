use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(19980801)],
    "To continue, please consult the code grid in the manual.  Enter the code at row 15, column 29.",
    vec![759716u64].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use crate::aoc::parse::trim;

    use super::*;
    use nom::{
        bytes::complete::{tag, take_until},
        sequence::preceded,
        Finish,
    };
    use std::str::FromStr;

    /// Defines the problem, which can be parsed from text input.
    pub struct Problem {
        /// Row of the manual number we need to calculate (starts at 1).
        row: u64,
        /// Column of the manual number we need to calculate (starts at 1).
        col: u64,
    }
    impl FromStr for Problem {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let row = preceded::<_, _, _, NomParseError, _, _>(
                take_until("row"),
                preceded(tag("row"), trim(false, nom::character::complete::u64)),
            )(s)
            .finish()
            .discard_input()?;

            let col = preceded::<_, _, _, NomParseError, _, _>(
                take_until("column"),
                preceded(tag("column"), trim(false, nom::character::complete::u64)),
            )(s)
            .finish()
            .discard_input()?;

            Ok(Problem { row, col })
        }
    }

    impl Problem {
        /// Solve a part of the problem by calculating the number at the correct position.
        pub fn solve(&self) -> AocResult<u64> {
            let (row, col) = (self.row, self.col);

            // Calculate the number in the sequence of codes from the table coordinates.
            // See the notes for a derivation of this formula.
            // TODO: Derive formula in the notes.
            let n = ((col - 1) * (col + 2) + (2 * col + row - 2) * (row - 1)) / 2;

            let mut x = 20151125;
            for _ in 0..n {
                x = (252533 * x) % 33554393;
            }

            Ok(x)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 25,
    name: "Let It Snow",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let problem: Problem = input.expect_input()?.parse()?;

            // Process
            Ok(problem.solve()?.into())
        },
    ],
};
