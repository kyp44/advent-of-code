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

#[derive(Debug)]
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

pub const SOLUTION: Solution = Solution {
    day: 22,
    name: "Reactor Reboot",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let steps = RebootStep::gather(input.expect_input()?.lines())?;

            let mut sizes: Vec<u64> = Vec::new();
            for i in 0..3 {
                let a = steps
                    .iter()
                    .map(|s| s.cuboid().ranges[i].start())
                    .min()
                    .unwrap();
                let b = steps
                    .iter()
                    .map(|s| s.cuboid().ranges[i].end())
                    .max()
                    .unwrap();
                let s = b - a + 1;
                println!("TODO: {} {} {}", a, b, s);
                sizes.push(s.try_into().unwrap());
            }
            println!("Elements: {}", sizes.into_iter().product::<u64>());

            // Process
            Ok(0u64.into())
        },
    ],
};
