use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2";
            answers = unsigned![123];
        }
        actual_answers = unsigned![123];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use cgmath::{Point2, Vector2};
    use nom::{
        branch::alt, bytes::complete::tag, character::complete::space1, combinator::map,
        sequence::separated_pair,
    };

    enum Direction {
        Left,
        Right,
        Up,
        Down,
    }
    impl Parsable<'_> for Direction {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            alt((
                map(tag("L"), |_| Self::Left),
                map(tag("R"), |_| Self::Right),
                map(tag("U"), |_| Self::Up),
                map(tag("D"), |_| Self::Down),
            ))(input)
        }
    }
    impl Direction {
        pub fn as_vector(&self) -> Vector2<isize> {
            match self {
                Direction::Left => -Vector2::unit_x(),
                Direction::Right => Vector2::unit_x(),
                Direction::Up => Vector2::unit_y(),
                Direction::Down => -Vector2::unit_y(),
            }
        }
    }

    struct Move {
        direction: Direction,
        spaces: u8,
    }
    impl Parsable<'_> for Move {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(Direction::parser, space1, nom::character::complete::u8),
                |(direction, spaces)| Move { direction, spaces },
            )(input)
        }
    }

    struct Rope {
        head: Point2<isize>,
        tail: Point2<isize>,
    }
    impl Default for Rope {
        fn default() -> Self {
            Self {
                head: Point2::new(0, 0),
                tail: Point2::new(0, 0),
            }
        }
    }
    impl Rope {
        fn move_single(&mut self, direction: Direction) {
            self.head += direction.as_vector();

            // The tail needs to catchup
            todo!()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 9,
    name: "Rope Bridge",
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
