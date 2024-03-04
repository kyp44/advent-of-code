use aoc::prelude::*;
use euclid::{
    default::{Point2D, Vector2D},
    point2, vec2,
};
use std::str::FromStr;

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
    use nom::{character::complete::one_of, combinator::map, sequence::pair};
    use std::fmt::Debug;

    /// A single navigation instruction, which can be parsed from text input.
    #[derive(Debug)]
    enum Instruction {
        /// Move the ship or waypoint by some relative displacement.
        Move(Vector2D<i32>),
        /// Turn by some angle or rotate the waypoint about the ship.
        Turn(i32),
        /// Move forward in the currently facing direction or to the waypoint.
        Forward(i32),
    }
    impl Parsable<'_> for Instruction {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                pair(one_of("NSEWLRF"), nom::character::complete::i32),
                |(c, n)| {
                    use Instruction::*;
                    match c {
                        'N' => Move(vec2(0, 1) * n),
                        'S' => Move(vec2(0, -1) * n),
                        'E' => Move(vec2(1, 0) * n),
                        'W' => Move(vec2(-1, 0) * n),
                        'L' => Turn(n / 90),
                        'R' => Turn(-n / 90),
                        'F' => Forward(n),
                        _ => panic!(),
                    }
                },
            )(input.trim())
        }
    }

    impl Instruction {
        /// Gets the new facing direction given the current one and turn distance.
        fn turn(facing: i32, turn: i32) -> i32 {
            (facing + turn).rem_euclid(4)
        }

        /// Gets translation vector given facing direction and distance.
        fn go_forward(facing: i32, distance: i32) -> Vector2D<i32> {
            let vec = match facing % 4 {
                0 => vec2(1, 0),
                1 => vec2(0, 1),
                2 => vec2(-1, 0),
                3 => vec2(0, -1),
                _ => panic!(),
            };
            vec * distance
        }

        /// Rotates a point given a turn number.
        fn rotate_point(turn: i32, point: &Point2D<i32>) -> Point2D<i32> {
            match Instruction::turn(0, turn) {
                0 => *point,
                1 => point2(-point.y, point.x),
                2 => -*point,
                3 => point2(point.y, -point.x),
                _ => panic!(),
            }
        }
    }

    /// A set of navigation instructions, which can be parsed from text input.
    pub struct NavigationInstructions {
        /// The list of instructions.
        instructions: Vec<Instruction>,
    }
    impl FromStr for NavigationInstructions {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                instructions: Instruction::gather(s.lines())?,
            })
        }
    }
    impl NavigationInstructions {
        /// Follows the instructions to determine the ship's final position.
        ///
        /// Optionally pass an initial waypoint location relative to the ship.
        /// If no initial waypoint is specified the commands always act on the ship,
        /// otherwise most commands act on the waypoint.
        pub fn final_ship_position(&self, initial_waypoint: Option<&Point2D<i32>>) -> Point2D<i32> {
            let mut position = Point2D::zero();
            match initial_waypoint {
                None => {
                    let mut facing = 0;
                    for inst in self.instructions.iter() {
                        match inst {
                            Instruction::Move(dv) => position += *dv,
                            Instruction::Turn(a) => facing = Instruction::turn(facing, *a),
                            Instruction::Forward(d) => {
                                position += Instruction::go_forward(facing, *d)
                            }
                        }
                        //println!("Instruction: {:?}, Facing: {:?}, Position {:?}", inst, facing, position);
                    }
                }
                Some(wp) => {
                    let mut waypoint = *wp;
                    for inst in self.instructions.iter() {
                        match inst {
                            Instruction::Move(dv) => waypoint += *dv,
                            Instruction::Turn(a) => {
                                waypoint = Instruction::rotate_point(*a, &waypoint)
                            }
                            Instruction::Forward(d) => position += waypoint.to_vector() * *d,
                        }
                        //println!("Instruction: {:?}, Waypoint: {:?}, Position {:?}", inst, waypoint, position);
                    }
                }
            }
            position
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 12,
    name: "Rain Risk",
    preprocessor: Some(|input| Ok(Box::new(NavigationInstructions::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<NavigationInstructions>()?
                    .final_ship_position(None)
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
                    .expect_data::<NavigationInstructions>()?
                    .final_ship_position(Some(&point2(10, 1)))
                    .to_vector()
                    .manhattan_len()
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
