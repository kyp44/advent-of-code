use std::{collections::HashSet, convert::TryInto};

use cgmath::{Vector2, Zero};
use nom::{character::complete::one_of, combinator::map, multi::many1};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
        vec![Unsigned(2565), Unsigned(2639)],
    ">",
    vec![Some(Unsigned(2)), None],
    "^v",
    vec![None, Some(Unsigned(3))],
    "^>v<",
    vec![4u64, 3].answer_vec(),
    "^v^v^v^v^v",
    vec![2u64, 11].answer_vec()
    }
}

type Point = Vector2<i32>;
enum Direction {
    North,
    East,
    South,
    West,
}
impl Parseable<'_> for Direction {
    fn parser(input: &str) -> NomParseResult<Self> {
        use Direction::*;
        map(one_of("^>v<"), |s| match s {
            '^' => North,
            '>' => East,
            'v' => South,
            '<' => West,
            _ => panic!(),
        })(input)
    }
}
impl Direction {
    fn direction(&self) -> Point {
        use Direction::*;
        match self {
            North => Vector2::unit_y(),
            East => Vector2::unit_x(),
            South => -Vector2::unit_y(),
            West => -Vector2::unit_x(),
        }
    }
}

trait Part {
    fn visited_houses(directions: &[Direction]) -> HashSet<Point>;
}
struct PartA;
impl Part for PartA {
    fn visited_houses(directions: &[Direction]) -> HashSet<Point> {
        let mut vh: HashSet<Point> = directions
            .iter()
            .scan(Vector2::zero(), |a, d| {
                *a += d.direction();
                Some(*a)
            })
            .collect();
        vh.insert(Vector2::zero());
        vh
    }
}
struct PartB;
impl Part for PartB {
    fn visited_houses(directions: &[Direction]) -> HashSet<Point> {
        let mut vh = HashSet::new();
        vh.insert(Vector2::zero());
        let mut santa = Vector2::zero();
        let mut robo = Vector2::zero();
        let mut santa_turn = true;
        for dir in directions {
            if santa_turn {
                santa += dir.direction();
                vh.insert(santa);
            } else {
                robo += dir.direction();
                vh.insert(robo);
            }
            santa_turn = !santa_turn;
        }
        vh
    }
}

struct Houses {
    directions: Vec<Direction>,
}
impl Parseable<'_> for Houses {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(many1(Direction::parser), |directions| Houses { directions })(input)
    }
}
impl Houses {
    fn visited_houses<P: Part>(&self) -> HashSet<Point> {
        P::visited_houses(&self.directions)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 3,
    name: "Perfectly Spherical Houses in a Vacuum",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let houses = Houses::from_str(input)?;

            // Process
            Ok(Answer::Unsigned(
                houses.visited_houses::<PartA>().len().try_into().unwrap(),
            ))
        },
        // Part b)
        |input| {
            // Generation
            let houses = Houses::from_str(input)?;

            // Process
            Ok(Answer::Unsigned(
                houses.visited_houses::<PartB>().len().try_into().unwrap(),
            ))
        },
    ],
};
