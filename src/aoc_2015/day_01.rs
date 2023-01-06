use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Signed;

    solution_test! {
        vec![Signed(74), Signed(1795)],
        "(())",
    vec![0i64].answer_vec(),
    "()()",
    vec![0i64].answer_vec(),
    "(((",
    vec![3i64].answer_vec(),
    "(()(()(",
    vec![3i64].answer_vec(),
    "))(((((",
    vec![3i64].answer_vec(),
    "())",
    vec![-1i64].answer_vec(),
    "))(",
    vec![-1i64].answer_vec(),
    ")))",
    vec![-3i64].answer_vec(),
    ")())())",
    vec![-3i64].answer_vec(),
    ")",
    vec![None, Some(Signed(1))],
    "()())",
    vec![None, Some(Signed(5))]
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use nom::{character::complete::one_of, combinator::map, multi::many1};

    /// A direction in which Santa can go.
    enum Direction {
        /// Up a floor.
        Up,
        /// Down a floor.
        Down,
    }
    impl Parseable<'_> for Direction {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(one_of("()"), |c| match c {
                '(' => Direction::Up,
                ')' => Direction::Down,
                _ => panic!(),
            })(input)
        }
    }
    impl Direction {
        /// Change in floor number if Santa goes in this direction.
        fn floor_change(&self) -> i64 {
            match self {
                Direction::Up => 1,
                Direction::Down => -1,
            }
        }
    }

    /// A step by step list of directions that can be parsed from text input.
    pub struct Directions {
        /// The list of directions.
        directions: Box<[Direction]>,
    }
    impl Parseable<'_> for Directions {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(many1(Direction::parser), |v| Directions {
                directions: v.into_boxed_slice(),
            })(input)
        }
    }
    impl Directions {
        /// Returns an [Iterator] of floor numbers when the directions are followed,
        /// starting at floor 0.
        pub fn floors(&self) -> impl Iterator<Item = i64> + '_ {
            self.directions.iter().scan(0i64, |a, d| {
                *a += d.floor_change();
                Some(*a)
            })
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 1,
    name: "Not Quite Lisp",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let directions = Directions::from_str(input.expect_input()?)?;

            // Process
            Ok(directions.floors().last().unwrap().into())
        },
        // Part two
        |input| {
            // Generation
            let directions = Directions::from_str(input.expect_input()?)?;

            // Process
            let pos =
                directions.floors().position(|f| f < 0).ok_or_else(|| {
                    AocError::Process("Santa never goes into the basement".into())
                })? + 1;
            Ok(Answer::Signed(pos.try_into().unwrap()))
        },
    ],
};
