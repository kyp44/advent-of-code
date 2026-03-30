use aoc::prelude::*;
use euclid::{
    default::{Point2D, Vector2D},
    point2, vec2,
};

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "F10
N3
F7
R90
F11";
            answers = unsigned![25, 286];
        }
        actual_answers = unsigned![2228, 42908];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use bare_metal_modulo::MNum;
    use bare_metal_modulo::ModNumC;
    use nom::{character::complete::one_of, combinator::map, sequence::pair};
    use std::fmt::Debug;

    /// The position type of the ship and waypoint.
    type Point = Point2D<i32>;
    /// The vector type to add to [`Point`]s.
    type Vector = Vector2D<i32>;
    /// The numeric form of a [`Direction`].
    type DirectionNum = ModNumC<u8, 4>;
    /// A number of turns.
    type NumTurns = DirectionNum;

    /// A cardinal direction.
    #[derive(Clone, Copy, Debug)]
    pub enum Direction {
        /// Positive `y`.
        North,
        /// Negative `x`.
        West,
        /// Negative `y`.
        South,
        /// Positive `x`.
        East,
    }
    impl From<DirectionNum> for Direction {
        fn from(value: DirectionNum) -> Self {
            match value.a() {
                0 => Self::North,
                1 => Self::West,
                2 => Self::South,
                3 => Self::East,
                _ => unreachable!(),
            }
        }
    }
    impl From<Direction> for DirectionNum {
        fn from(value: Direction) -> Self {
            DirectionNum::new(match value {
                Direction::North => 0,
                Direction::West => 1,
                Direction::South => 2,
                Direction::East => 3,
            })
        }
    }
    impl Direction {
        /// Returns the new direction after turning `num_turns`
        /// counter-clockwise from this one.
        pub fn turn(&self, num_turns: NumTurns) -> Self {
            (DirectionNum::from(*self) + num_turns).into()
        }

        /// Returns the unit vector corresponding to this direction.
        pub fn as_vector(&self) -> Vector {
            match self {
                Direction::North => Vector::unit_y(),
                Direction::West => -Vector::unit_x(),
                Direction::South => -Vector::unit_y(),
                Direction::East => Vector::unit_x(),
            }
        }
    }

    /// A single navigation instruction, which can be parsed from text input.
    #[derive(Clone, Debug)]
    pub enum NavInstruction {
        /// Move the ship or waypoint by some relative displacement.
        Move(Vector),
        /// Turn by some number of turns or rotate the waypoint about the ship,
        /// both counter-clockwise.
        Turn(NumTurns),
        /// Move forward in the currently facing direction or to the waypoint.
        Forward(i32),
    }
    impl Parsable for NavInstruction {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(
                pair(one_of("NSEWLRF"), nom::character::complete::i32),
                |(c, n)| {
                    let n8 = u8::try_from(n / 90).unwrap();
                    match c {
                        'N' => NavInstruction::Move(vec2(0, 1) * n),
                        'S' => NavInstruction::Move(vec2(0, -1) * n),
                        'E' => NavInstruction::Move(vec2(1, 0) * n),
                        'W' => NavInstruction::Move(vec2(-1, 0) * n),
                        'L' => NavInstruction::Turn(NumTurns::new(n8)),
                        'R' => NavInstruction::Turn(-NumTurns::new(n8)),
                        'F' => NavInstruction::Forward(n),
                        _ => panic!(),
                    }
                },
            )
            .parse(input.trim())
        }
    }
    impl NavInstruction {
        /// Rotates a point counter-clockwise about the origin given a number of
        /// turns.
        fn rotate_point(num_turns: NumTurns, point: Point) -> Point {
            match num_turns.a() {
                0 => point,
                1 => point2(-point.y, point.x),
                2 => -point,
                3 => point2(point.y, -point.x),
                _ => unreachable!(),
            }
        }
    }
    impl Instruction for NavInstruction {
        type Registers = ShipState;
        type YieldItem = ();
        type Err = AocError;

        fn execute(
            &self,
            program_counter: Option<&mut ProgramCounter<Self>>,
            registers: &mut Self::Registers,
        ) -> Result<Self::YieldItem, Self::Err> {
            match registers {
                ShipState::Basic { facing, position } => match self {
                    NavInstruction::Move(dv) => *position += *dv,
                    NavInstruction::Turn(a) => *facing = facing.turn(*a),
                    NavInstruction::Forward(d) => *position += facing.as_vector() * *d,
                },
                ShipState::Waypoint {
                    waypoint,
                    ship: position,
                } => match self {
                    NavInstruction::Move(dv) => *waypoint += *dv,
                    NavInstruction::Turn(a) => *waypoint = Self::rotate_point(*a, *waypoint),
                    NavInstruction::Forward(d) => *position += waypoint.to_vector() * *d,
                },
            }
            program_counter.unwrap().increment();
            Ok(())
        }
    }

    /// A state for each part of the problem.
    pub enum ShipState {
        /// A basic state for part one.
        Basic {
            /// The direction the ship is facing.
            facing: Direction,
            /// The position of the ship.
            position: Point,
        },
        /// The ship and waypoint states for part two.
        Waypoint {
            /// The position of the waypoint.
            waypoint: Point,
            /// The position of the ship.
            ship: Point,
        },
    }
    impl ShipState {
        /// Returns the starting state for part one.
        pub fn starting_basic() -> Self {
            Self::Basic {
                facing: Direction::East,
                position: Point::zero(),
            }
        }

        /// Returns the starting state for part two.
        pub fn starting_waypoint() -> Self {
            Self::Waypoint {
                waypoint: Point::new(10, 1),
                ship: Point::zero(),
            }
        }

        /// Returns the ship position for either part.        
        pub fn ship_position(&self) -> Point {
            match self {
                ShipState::Basic {
                    facing: _,
                    position,
                } => *position,
                ShipState::Waypoint {
                    waypoint: _,
                    ship: position,
                } => *position,
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 12,
    name: "Rain Risk",
    preprocessor: Some(|input| Ok(Box::new(Program::<NavInstruction>::parse(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<Program<NavInstruction>>()?
                    .execute(ShipState::starting_basic())?
                    .into_registers()
                    .ship_position()
                    .to_vector()
                    .manhattan_len()
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<Program<NavInstruction>>()?
                    .execute(ShipState::starting_waypoint())?
                    .into_registers()
                    .ship_position()
                    .to_vector()
                    .manhattan_len()
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
