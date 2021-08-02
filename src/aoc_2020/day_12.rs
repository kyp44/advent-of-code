use crate::aoc::prelude::*;
use cgmath::Vector2;
use nom::{
    character::complete::{digit1, one_of},
    combinator::map,
    sequence::pair,
};
use num::Signed;
use std::convert::TryInto;
use std::fmt::Debug;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Number;

    solution_test! {
    vec![Number(2228), Number(42908)],
    "F10
N3
F7
R90
F11",
        vec![25, 286].answer_vec()
    }
}

trait Manhatten<T> {
    fn manhatten(&self) -> T;
}

impl<T, O> Manhatten<O> for Vector2<T>
where
    T: Signed + TryInto<O>,
    T::Error: Debug,
{
    fn manhatten(&self) -> O {
        (self.x.abs() + self.y.abs()).try_into().unwrap()
    }
}

#[derive(Debug)]
enum Instruction {
    Move(Vector2<i32>),
    Turn(i32),
    Forward(i32),
}

impl Parseable<'_> for Instruction {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(pair(one_of("NSEWLRF"), digit1), |(c, n): (char, &str)| {
            use Instruction::*;
            let n: i32 = n.parse().unwrap();
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
        })(input.trim())
    }
}

impl Instruction {
    /// Get new facing direction given the current one and turn distance
    fn turn(facing: i32, turn: i32) -> i32 {
        (facing + turn).rem_euclid(4)
    }

    /// Get translation vector given facing direction and distance
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

    /// Rotates a point given a turn number
    fn rotate_point(turn: i32, point: &Vector2<i32>) -> Vector2<i32> {
        match Instruction::turn(0, turn) {
            0 => *point,
            1 => Vector2::new(-point.y, point.x),
            2 => -*point,
            3 => Vector2::new(point.y, -point.x),
            _ => panic!(),
        }
    }
}

struct Ship;

impl Ship {
    fn follow_ship_instructions(instructions: &[Instruction]) -> Vector2<i32> {
        let mut facing = 0;
        let mut position = Vector2::new(0, 0);
        for inst in instructions.iter() {
            match inst {
                Instruction::Move(dv) => position += *dv,
                Instruction::Turn(a) => facing = Instruction::turn(facing, *a),
                Instruction::Forward(d) => position += Instruction::go_forward(facing, *d),
            }
            //println!("Insruction: {:?}, Facing: {:?}, Position {:?}", inst, facing, position);
        }
        position
    }

    fn follow_waypoint_instructions(
        instructions: &[Instruction],
        waypoint: &Vector2<i32>,
    ) -> Vector2<i32> {
        let mut waypoint = *waypoint;
        let mut position = Vector2::new(0, 0);
        for inst in instructions.iter() {
            match inst {
                Instruction::Move(dv) => waypoint += *dv,
                Instruction::Turn(a) => waypoint = Instruction::rotate_point(*a, &waypoint),
                Instruction::Forward(d) => position += *d * waypoint,
            }
            //println!("Insruction: {:?}, Waypoint: {:?}, Position {:?}", inst, waypoint, position);
        }
        position
    }
}

pub const SOLUTION: Solution = Solution {
    day: 12,
    name: "Rain Risk",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let instructions = Instruction::gather(input.lines())?;

            // Process
            Ok(Answer::Number(
                Ship::follow_ship_instructions(&instructions).manhatten(),
            ))
        },
        // Part b)
        |input| {
            // Generation
            let instructions = Instruction::gather(input.lines())?;

            // Process
            Ok(Answer::Number(
                Ship::follow_waypoint_instructions(&instructions, &Vector2::new(10, 1)).manhatten(),
            ))
        },
    ],
};
