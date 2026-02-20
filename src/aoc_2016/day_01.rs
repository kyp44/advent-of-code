use aoc::prelude::*;

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
    use itertools::Itertools;
    use nom::{branch::alt, bytes::complete::tag, combinator::map, sequence::pair};
    use std::collections::HashSet;

    use super::*;

    /// The vector type used for intersection positions between blocks where (0,
    /// 0) is the starting position.
    type Vector = euclid::default::Vector2D<i32>;

    /// A direction to turn.
    ///
    /// Can be parsed from text input.
    #[derive(Clone, Copy, Debug)]
    pub enum TurnDirection {
        /// Turn to the left.
        Left,
        /// Turn to the right.
        Right,
    }
    impl Parsable for TurnDirection {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            alt((
                map(tag("L"), |_| Self::Left),
                map(tag("R"), |_| Self::Right),
            ))
            .parse(input)
        }
    }

    /// An instruction to turn and then walk some number of blocks.
    ///
    /// Can be parsed from text input.
    #[derive(Debug)]
    pub struct WalkingInstruction {
        /// The direction to turn before walking.
        pub turn_direction: TurnDirection,
        /// The number of blocks to walk after turning.
        pub distance: u16,
    }
    impl Parsable for WalkingInstruction {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
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
    impl Instruction for WalkingInstruction {
        type Registers = Location;
        type YieldItem = Vector;
        type Error = AocError;

        fn execute(
            &self,
            registers: &mut Self::Registers,
        ) -> Result<Executed<Self::YieldItem>, Self::Error> {
            registers.facing = registers.facing.turn(self.turn_direction);
            registers.position += registers.facing.as_vector() * i32::from(self.distance);

            Ok(Executed::no_jump(registers.position))
        }
    }

    /// A cardinal direction.
    #[derive(Clone, Copy, Debug, Default)]
    enum Direction {
        /// North, or positive `y`.
        #[default]
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

        /// Returns a vector corresponding to walking one block in this
        /// direction.
        pub fn as_vector(&self) -> Vector {
            match self {
                Direction::North => Vector::unit_y(),
                Direction::East => Vector::unit_x(),
                Direction::South => -Vector::unit_y(),
                Direction::West => -Vector::unit_x(),
            }
        }

        /// Returns the direction and distance given a direction vector.
        pub fn from_vector(v: Vector) -> Option<(Self, u16)> {
            if v.y == 0 {
                if v.x > 0 {
                    Some(Self::East)
                } else if v.x < 0 {
                    Some(Self::West)
                } else {
                    None
                }
                .map(|dir| (dir, v.x.abs().try_into().unwrap()))
            } else if v.x == 0 {
                if v.y > 0 {
                    Some(Self::North)
                } else if v.y < 0 {
                    Some(Self::South)
                } else {
                    None
                }
                .map(|dir| (dir, v.y.abs().try_into().unwrap()))
            } else {
                None
            }
        }

        /// Returns an [`Iterator`] over every intersection passed through when
        /// walking this direction `distance` blocks from the
        /// `starting_position`.
        ///
        /// NOTE: The `starting_position` is not the first item, which is one
        /// block in this direction.
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

    /// Our current location.
    #[derive(Default)]
    pub struct Location {
        /// The position of the block where we are.
        position: Vector,
        /// The direction we are facing.
        facing: Direction,
    }

    /// Executes the instructions and returns the first intersection that is
    /// visited twice, or `None` if no intersection is ever visited twice.
    ///
    /// NOTE: This counts all intersections walked through, not just
    /// intersections at the end of each step.
    pub fn first_visited_twice(program: &Program<WalkingInstruction>) -> Option<Vector> {
        let initial_location = Location::default();
        let mut visited = HashSet::<Vector>::new();

        visited.insert(initial_location.position);
        for (start, end) in (std::iter::once(initial_location.position)
            .chain(program.executor(initial_location).map(|r| r.unwrap())))
        .tuple_windows()
        {
            // Walk along the path block by block, adding each position to the visited set
            let (dir, dist) = Direction::from_vector(end - start)?;

            for pos in dir.every_block(start, dist) {
                // Have we been here before?
                if !visited.insert(pos) {
                    return Some(pos);
                }
            }
        }

        None
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 1,
    name: "No Time for a Taxicab",
    preprocessor: Some(|input| {
        Ok(Box::new(Program::<WalkingInstruction>::new(
            WalkingInstruction::from_csv(input)?,
        ))
        .into())
    }),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(u64::try_from(
                input
                    .expect_data::<Program<WalkingInstruction>>()?
                    .execute(Location::default())?
                    .last_yielded
                    .unwrap()
                    .manhattan_len(),
            )
            .unwrap()
            .into())
        },
        // Part two
        |input| {
            // Process
            Ok(u64::try_from(
                first_visited_twice(input.expect_data::<Program<WalkingInstruction>>()?)
                    .ok_or(AocError::NoSolution)?
                    .manhattan_len(),
            )
            .unwrap()
            .into())
        },
    ],
};
