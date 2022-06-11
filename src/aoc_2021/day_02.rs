use std::str::FromStr;

use nom::{branch::alt, combinator::map};

use crate::aoc::{parse::field_line_parser, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
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

enum Direction {
    Forward(u8),
    Down(u8),
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

trait Part {
    fn apply_direction(&mut self, direction: &Direction);
    fn new() -> Self;
}

struct PartA {
    horizontal: i64,
    depth: i64,
}

struct PartB {
    horizontal: i64,
    depth: i64,
    aim: i64,
}

impl Part for PartA {
    fn new() -> Self {
        Self {
            horizontal: 0,
            depth: 0,
        }
    }

    fn apply_direction(&mut self, direction: &Direction) {
        match direction {
            Direction::Forward(n) => self.horizontal += i64::from(*n),
            Direction::Down(n) => self.depth += i64::from(*n),
            Direction::Up(n) => self.depth -= i64::from(*n),
        }
    }
}

impl Part for PartB {
    fn new() -> Self {
        Self {
            horizontal: 0,
            depth: 0,
            aim: 0,
        }
    }

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
}

struct Course {
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
    fn end_position<P: Part>(&self) -> P {
        let mut position = P::new();
        for direction in self.directions.iter() {
            position.apply_direction(direction)
        }
        position
    }
}

pub const SOLUTION: Solution = Solution {
    day: 2,
    name: "Dive!",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let course = Course::from_str(input.expect_input()?)?;
            let end_position = course.end_position::<PartA>();

            // Process
            Ok((end_position.horizontal * end_position.depth).into())
        },
        // Part b)
        |input| {
            // Generation
            let course = Course::from_str(input.expect_input()?)?;
            let end_position = course.end_position::<PartB>();

            // Process
            Ok((end_position.horizontal * end_position.depth).into())
        },
    ],
};
