use nom::{
    bytes::complete::{tag, take_until},
    sequence::preceded,
    Finish,
};
use std::str::FromStr;

use crate::aoc::{parse::trim, prelude::*};

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

struct Problem {
    row: u64,
    col: u64,
}
impl FromStr for Problem {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let row = preceded::<_, _, _, NomParseError, _, _>(
            take_until("row"),
            preceded(tag("row"), trim(nom::character::complete::u64)),
        )(s)
        .finish()
        .discard_input()?;

        let col = preceded::<_, _, _, NomParseError, _, _>(
            take_until("column"),
            preceded(tag("column"), trim(nom::character::complete::u64)),
        )(s)
        .finish()
        .discard_input()?;

        Ok(Problem { row, col })
    }
}

impl Problem {
    fn solve(&self) -> AocResult<u64> {
        // Calculate the number in the sequence of codes.
        // This formula was derived mathematically for the diagonal table in
        // the problem description, but starting at 0 at (1,1) instead of 1.
        let (row, col) = (self.row, self.col);
        let n = ((col - 1) * (col + 2) + (2 * col + row - 2) * (row - 1)) / 2;

        let mut x = 20151125;
        for _ in 0..n {
            x = (252533 * x) % 33554393;
        }

        Ok(x)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 25,
    name: "Let It Snow",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let problem: Problem = input.parse()?;

            // Process
            Ok(problem.solve()?.into())
        },
    ],
};
