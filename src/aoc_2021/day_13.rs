use std::{collections::HashSet, str::FromStr, fmt::Debug};

use cgmath::Vector2;
use nom::{combinator::map, sequence::separated_pair, bytes::complete::tag};

use crate::aoc::{prelude::*, parse::trim};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
        vec![Unsigned(123)],
    "6,10
0,14
9,10
0,3
10,4
4,11
6,0
6,12
4,1
0,13
10,12
3,4
3,0
8,4
1,10
2,14
8,10
9,0

fold along y=7
fold along x=5",
        vec![12u64].answer_vec()
    }
}

#[derive(PartialEq, Eq, Hash)]
struct Point {
    point: Vector2<isize>,
}
impl Parseable<'_> for Point {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(
            separated_pair(
                nom::character::complete::i32,
                trim(tag(",")),
                nom::character::complete::i32,
            ),
            |(x, y)| Self {
                point: Vector2::new(x.try_into().unwrap(), y.try_into().unwrap())
            }
        )(input)
    }
}

struct Page {
    dots: HashSet<Point>,
}
impl FromStr for Page {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Page {
            dots: Point::gather(s.lines())?.into_iter().collect(),
        })
    }
}
impl Debug for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let grid = Grid::from_coordinates(self.dots.iter().map(|p| p.point));
        writeln!(f, "{:?}", "quaggles")
    }
}


enum Fold {
    Vertical(usize),
    Horizontal(usize),
}

pub const SOLUTION: Solution = Solution {
    day: 13,
    name: "Transparent Origami",
    solvers: &[
        // Part a)
        |input| {
            // Generation

            // Process
            Ok(0u64.into())
        },
    ],
};
