use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "To continue, please consult the code grid in the manual.  Enter the code at row 6, column 6.";
            answers = unsigned![27995004];
        }
        actual_answers = unsigned![19980801];
    }
}

/// Contains solution implementation items.
mod solution {
    use aoc::parse::trim;
    use bare_metal_modulo::{MNum, ModNumC};

    use super::*;
    use nom::{
        Finish,
        bytes::complete::{tag, take_until},
        sequence::preceded,
    };

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
            let row = preceded::<_, _, NomParseError, _, _>(
                take_until("row"),
                preceded(tag("row"), trim(false, nom::character::complete::u64)),
            )
            .parse(s)
            .finish()
            .discard_input()?;

            let col = preceded::<_, _, NomParseError, _, _>(
                take_until("column"),
                preceded(tag("column"), trim(false, nom::character::complete::u64)),
            )
            .parse(s)
            .finish()
            .discard_input()?;

            Ok(Problem { row, col })
        }
    }

    impl Problem {
        /// Solves a part of the problem by calculating the number at the
        /// correct position.
        pub fn solve(&self) -> AocResult<u64> {
            let (row, col) = (self.row, self.col);

            // Calculate the number in the sequence of codes from the table coordinates.
            // See the notes for a derivation of this formula.
            let n = ((col + row).pow(2) - col - 3 * row + 2) / 2;

            // Now calculate the nth number in the sequence.
            Ok((1..n)
                .fold(ModNumC::<u64, 33554393>::new(20151125), |x, _| x * 252533)
                .a())
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
            let problem: Problem = input.expect_text()?.parse()?;

            // Process
            Ok(problem.solve()?.into())
        },
    ],
};
