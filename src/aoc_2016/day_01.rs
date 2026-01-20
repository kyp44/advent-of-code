use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use Answer::Unsigned;
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "R2, L3";
            answers = unsigned![5];
        }
        example {
            input = "R2, R2, R2";
            answers = unsigned![2];
        }
        example {
            input = "R5, L5, R5, R3";
            answers = unsigned![12];
        }
        example {
            input = "R8, R4, R4, R8";
            answers = &[None, Some(Unsigned(4))];
        }
        actual_answers = unsigned![161, 110];
    }
}

/// Contains solution implementation items.
mod solution {
    use aoc::parse::trim;
    use nom::{branch::alt, bytes::complete::tag, combinator::map, sequence::pair};
    use std::{collections::HashSet, str::FromStr};

    use super::*;

    /// The vector type used for intersection positions between blocks where (0, 0) is the starting position.
    type Vector = euclid::default::Vector2D<i32>;

    /// A direction to turn.
    ///
    /// Can be parsed from text input.
    #[derive(Clone, Copy)]
    enum TurnDirection {
        /// Turn to the left.
        Left,
        /// Turn to the right.
        Right,
    }
    impl<'a> Parsable<'a> for TurnDirection {
        fn parser(input: &'a str) -> NomParseResult<&'a str, Self> {
            alt((
                map(tag("L"), |_| Self::Left),
                map(tag("R"), |_| Self::Right),
            ))
            .parse(input)
        }
    }

    /// A step in the [`Instructions`].
    ///
    /// Can be parsed from text input.
    struct Step {
        /// The direction to turn before walking.
        pub turn_direction: TurnDirection,
        /// The distance to walk after turning.
        pub distance: u16,
    }
    impl<'a> Parsable<'a> for Step {
        fn parser(input: &'a str) -> NomParseResult<&'a str, Self> {
            map(
                trim(
                    false,
                    pair(TurnDirection::parser, nom::character::complete::u16),
                ),
                |(turn_direction, distance)| Self {
                    turn_direction,
                    distance,
                },
            )
            .parse(input)
        }
    }

    /// A cardinal direction.
    #[derive(Clone, Copy)]
    enum Direction {
        /// North, or positive `y`.
        North,
        /// East, or positive `x`.
        East,
        /// South, or negative `y`.
        South,
        /// West, or negative `x`.
        West,
    }
    impl From<u8> for Direction {
        fn from(value: u8) -> Self {
            match value % 4 {
                0 => Self::North,
                1 => Self::East,
                2 => Self::South,
                3 => Self::West,
                _ => unreachable!(),
            }
        }
    }
    impl From<Direction> for u8 {
        fn from(value: Direction) -> Self {
            match value {
                Direction::North => 0,
                Direction::East => 1,
                Direction::South => 2,
                Direction::West => 3,
            }
        }
    }
    impl Direction {
        /// Applies a turn to face a new direction when facing this direction.
        pub fn turn(self, turn_direction: TurnDirection) -> Self {
            let mut dir: u8 = self.into();
            dir = match turn_direction {
                TurnDirection::Left => dir.wrapping_sub(1),
                TurnDirection::Right => dir.wrapping_add(1),
            };
            Self::from(dir)
        }

        /// Returns a vector corresponding to walking one block in this direction.
        pub fn as_vector(&self) -> Vector {
            match self {
                Direction::North => Vector::unit_y(),
                Direction::East => Vector::unit_x(),
                Direction::South => -Vector::unit_y(),
                Direction::West => -Vector::unit_x(),
            }
        }

        /// Returns an [`Iterator`] over every intersection passed through when walking
        /// this direction `distance` blocks from the `starting_position`.
        ///
        /// NOTE: The `starting_position` is not the first item, which is one block
        /// in this direction.
        pub fn every_block(
            &self,
            starting_position: Vector,
            distance: u16,
        ) -> impl Iterator<Item = Vector> + 'static {
            let direction: Vector = self.as_vector();
            let distance: i32 = distance.into();

            (1..=distance).map(move |d| starting_position + direction * d)
        }
    }

    /// A set of [`Step`]s in order.
    ///
    /// Can be parsed from text input.
    pub struct Instructions {
        /// The list of steps.
        steps: Vec<Step>,
    }
    impl FromStr for Instructions {
        type Err = NomParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                steps: Step::from_csv(s)?,
            })
        }
    }
    impl Instructions {
        /// Executes the instructions and return the final intersection at the end.
        pub fn final_position(&self) -> Vector {
            let mut position = Vector::zero();
            let mut direction = Direction::North;

            for step in self.steps.iter() {
                direction = direction.turn(step.turn_direction);
                position += direction.as_vector() * i32::from(step.distance);
            }

            position
        }

        /// Executes the instructions and returns the first intersection that is
        /// visited twice, or `None` if no intersection is ever visited twice.
        ///
        /// NOTE: This counts all intersections walked through, not just intersections
        /// at the end of each step.
        pub fn first_visited_twice(&self) -> Option<Vector> {
            let mut position = Vector::zero();
            let mut visited = HashSet::<Vector>::new();
            let mut direction = Direction::North;

            visited.insert(position);
            for step in self.steps.iter() {
                // Walk along the path block by block, adding each position to the visited set
                direction = direction.turn(step.turn_direction);

                for pos in direction.every_block(position, step.distance) {
                    // Have we been here before?
                    if visited.replace(pos).is_some() {
                        return Some(pos);
                    }
                    position = pos;
                }
            }

            None
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 1,
    name: "No Time for a Taxicab",
    preprocessor: Some(|input| Ok(Box::new(Instructions::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(u64::try_from(
                input
                    .expect_data::<Instructions>()?
                    .final_position()
                    .manhattan_len(),
            )
            .unwrap()
            .into())
        },
        // Part two
        |input| {
            // Process
            Ok(u64::try_from(
                input
                    .expect_data::<Instructions>()?
                    .first_visited_twice()
                    .ok_or(AocError::NoSolution)?
                    .manhattan_len(),
            )
            .unwrap()
            .into())
        },
    ],
};
