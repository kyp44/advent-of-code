use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Signed;

    solution_test! {
    vec![Signed(2102357), Signed(2101031224)],
    "forward 5
down 5
forward 8
up 3
down 8
forward 2",
    vec![150i64, 900].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::field_line_parser;
    use nom::{branch::alt, combinator::map};

    /// A direction the submarine can move, depending on the problem part.
    /// This can be parsed from text input.
    pub enum Direction {
        /// Increase horizontal position (part one) or both this and depth (part two) by some amount.
        Forward(u8),
        /// Increase depth (part one) or aim (part two) by some amount.
        Down(u8),
        /// Decrease depth (part one) or aim (part two) by some amount.
        Up(u8),
    }
    impl Parseable<'_> for Direction {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            alt((
                map(
                    field_line_parser("forward", nom::character::complete::u8),
                    Direction::Forward,
                ),
                map(
                    field_line_parser("down", nom::character::complete::u8),
                    Direction::Down,
                ),
                map(
                    field_line_parser("up", nom::character::complete::u8),
                    Direction::Up,
                ),
            ))(input)
        }
    }

    /// Behavior specific to one particular part of the problem, which
    /// holds the state of the submarine for the part.
    pub trait State: Default {
        /// Apply a direction to the state.
        fn apply_direction(&mut self, direction: &Direction);
        /// Calculate the result based on the current state.
        fn calculate_result(&self) -> i64;
    }

    /// Submarine state for part one.
    #[derive(Default)]
    pub struct StateOne {
        /// Submarine horizontal position.
        pub horizontal: i64,
        /// Submarine depth.
        pub depth: i64,
    }
    impl State for StateOne {
        fn apply_direction(&mut self, direction: &Direction) {
            match direction {
                Direction::Forward(n) => self.horizontal += i64::from(*n),
                Direction::Down(n) => self.depth += i64::from(*n),
                Direction::Up(n) => self.depth -= i64::from(*n),
            }
        }

        fn calculate_result(&self) -> i64 {
            self.horizontal * self.depth
        }
    }

    /// Submarine state for part two.
    #[derive(Default)]
    pub struct StateTwo {
        /// Submarine horizontal position.
        pub horizontal: i64,
        /// Submarine depth.
        pub depth: i64,
        /// Submarine aim.
        aim: i64,
    }
    impl State for StateTwo {
        fn apply_direction(&mut self, direction: &Direction) {
            match direction {
                Direction::Forward(n) => {
                    let n = i64::from(*n);
                    self.horizontal += n;
                    self.depth += self.aim * n;
                }
                Direction::Down(n) => self.aim += i64::from(*n),
                Direction::Up(n) => self.aim -= i64::from(*n),
            }
        }

        fn calculate_result(&self) -> i64 {
            self.horizontal * self.depth
        }
    }

    /// The course that the submarine takes, which can be parsed from text input.
    pub struct Course {
        /// Ordered list of directions to follow.
        directions: Box<[Direction]>,
    }
    impl FromStr for Course {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Course {
                directions: Direction::gather(s.lines())?.into_boxed_slice(),
            })
        }
    }
    impl Course {
        /// Return the submarine state after following all of the directions.
        pub fn end_position<P: State>(&self) -> P {
            let mut position = P::default();
            for direction in self.directions.iter() {
                position.apply_direction(direction)
            }
            position
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 2,
    name: "Dive!",
    preprocessor: Some(|input| Ok(Box::new(Course::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Course>()?
                .end_position::<StateOne>()
                .calculate_result()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Course>()?
                .end_position::<StateTwo>()
                .calculate_result()
                .into())
        },
    ],
};
