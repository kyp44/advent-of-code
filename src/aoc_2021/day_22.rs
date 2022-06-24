use std::ops::RangeInclusive;

use cgmath::Vector3;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space1,
    combinator::map,
    sequence::{separated_pair, terminated, tuple},
};

use crate::aoc::{parse::field_line_parser, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(123)],
    "on x=10..12,y=10..12,z=10..12
on x=11..13,y=11..13,z=11..13
off x=9..11,y=9..11,z=9..11
on x=10..10,y=10..10,z=10..10",
    vec![39u64].answer_vec()
    }
}

#[derive(Debug, Clone)]
struct Cuboid {
    ranges: Vector3<RangeInclusive<i32>>,
}
impl Parseable<'_> for Cuboid {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        fn parse_range(input: &str) -> NomParseResult<&str, RangeInclusive<i32>> {
            map(
                separated_pair(
                    nom::character::complete::i32,
                    tag(".."),
                    nom::character::complete::i32,
                ),
                |(a, b)| a..=b,
            )(input)
        }

        map(
            tuple((
                field_line_parser("x=", terminated(parse_range, tag(","))),
                field_line_parser("y=", terminated(parse_range, tag(","))),
                field_line_parser("z=", parse_range),
            )),
            |(x, y, z)| Self {
                ranges: Vector3::new(x, y, z),
            },
        )(input)
    }
}
impl Cuboid {
    fn intersection(&self, other: &Cuboid) -> Option<Cuboid> {
        let rx = self.ranges.x.intersection(&other.ranges.x);
        let ry = self.ranges.y.intersection(&other.ranges.y);
        let rz = self.ranges.z.intersection(&other.ranges.z);

        match (rx, ry, rz) {
            (Some(x), Some(y), Some(z)) => Some(Cuboid {
                ranges: Vector3::new(x, y, z),
            }),
            _ => None,
        }
    }

    fn num_points(&self) -> u64 {
        let ranges = &self.ranges;
        [&ranges.x, &ranges.y, &ranges.z]
            .into_iter()
            .map(|r| u64::try_from(r.len()).unwrap())
            .product::<u64>()
    }
}

#[derive(Debug)]
enum RebootStep {
    On(Cuboid),
    Off(Cuboid),
}
impl Parseable<'_> for RebootStep {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        map(
            separated_pair(alt((tag("on"), tag("off"))), space1, Cuboid::parser),
            |(w, cub)| match w {
                "on" => Self::On(cub),
                _ => Self::Off(cub),
            },
        )(input)
    }
}
impl RebootStep {
    fn cuboid(&self) -> &Cuboid {
        match self {
            RebootStep::On(c) => c,
            RebootStep::Off(c) => c,
        }
    }
}

#[derive(Debug)]
enum Set {
    Empty,
    Basic(Cuboid),
    Difference(Box<Set>, Box<Set>),
    Union(Box<Set>, Box<Set>),
}
impl Set {
    fn intersection(&self, other: &Self) -> Self {
        match self {
            Set::Empty => Self::Empty,
            Set::Basic(cs) => match other {
                Set::Empty => Self::Empty,
                Set::Basic(co) => match cs.intersection(&co) {
                    Some(cf) => Self::Basic(cf),
                    None => Self::Empty,
                },
                Set::Difference(_, _) => todo!(),
                Set::Union(_, _) => todo!(),
            },
            Set::Difference(_, _) => todo!(),
            Set::Union(_, _) => todo!(),
        }
    }

    fn num_points(&self) -> u64 {
        match self {
            Set::Empty => 0,
            Set::Basic(c) => c.num_points(),
            Set::Difference(_, _) => todo!(),
            Set::Union(_, _) => todo!(),
        }
    }
}
impl From<Cuboid> for Set {
    fn from(c: Cuboid) -> Self {
        Self::Basic(c)
    }
}
impl FromIterator<RebootStep> for Set {
    fn from_iter<T: IntoIterator<Item = RebootStep>>(iter: T) -> Self {
        todo!()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 22,
    name: "Reactor Reboot",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let steps = RebootStep::gather(input.expect_input()?.lines())?;

            // Process
            Ok(0u64.into())
        },
    ],
};
