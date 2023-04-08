use cgmath::Vector2;
use cgmath::{EuclideanSpace, Point2};
use std::str::FromStr;

use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "F10
N3
F7
R90
F11";
            answers = vec![25u64, 286].answer_vec();
        }
        actual_answers = vec![Unsigned(2228), Unsigned(42908)];
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
        Move(Vector2<i32>),
        /// Turn by some angle or rotate the waypoint about the ship.
        Turn(i32),
        /// Move forward in the currently facing direction or to the waypoint.
        Forward(i32),
    }
    impl Parseable<'_> for Instruction {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                pair(one_of("NSEWLRF"), nom::character::complete::i32),
                |(c, n)| {
                    use Instruction::*;
                    match c {
                        'N' => Move(n * Vector2::unit_y()),
                        'S' => Move(-n * Vector2::unit_y()),
                        'E' => Move(n * Vector2::unit_x()),
                        'W' => Move(-n * Vector2::unit_x()),
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
        fn go_forward(facing: i32, distance: i32) -> Vector2<i32> {
            distance
                * match facing % 4 {
                    0 => Vector2::unit_x(),
                    1 => Vector2::unit_y(),
                    2 => -Vector2::unit_x(),
                    3 => -Vector2::unit_y(),
                    _ => panic!(),
                }
        }

        /// Rotates a point given a turn number.
        fn rotate_point(turn: i32, point: &Point2<i32>) -> Point2<i32> {
            match Instruction::turn(0, turn) {
                0 => *point,
                1 => Point2::new(-point.y, point.x),
                2 => Point2::from_vec(-point.to_vec()),
                3 => Point2::new(point.y, -point.x),
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
        pub fn final_ship_position(&self, initial_waypoint: Option<&Point2<i32>>) -> Point2<i32> {
            let mut position = PointExt::origin();
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
                            Instruction::Forward(d) => position += *d * waypoint.to_vec(),
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
                    .to_vec()
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
                    .final_ship_position(Some(&Point2::new(10, 1)))
                    .to_vec()
                    .manhattan_len()
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
