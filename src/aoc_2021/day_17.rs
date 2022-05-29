use std::ops::RangeInclusive;

use nom::{bytes::complete::tag, combinator::map, sequence::separated_pair};

use crate::aoc::{
    parse::{field_line_parser, trim},
    prelude::*,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(123)],
    "target area: x=20..30, y=-10..-5",
    vec![123u64].answer_vec()
    }
}

#[derive(Debug)]
struct TargetArea {
    range_x: RangeInclusive<i32>,
    range_y: RangeInclusive<i32>,
}
impl Parseable<'_> for TargetArea {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        fn range_parser(
            label: &'static str,
        ) -> impl FnMut(&str) -> NomParseResult<&str, RangeInclusive<i32>> {
            move |input| {
                map(
                    field_line_parser(
                        label,
                        separated_pair(
                            nom::character::complete::i32,
                            tag(".."),
                            nom::character::complete::i32,
                        ),
                    ),
                    |(a, b)| a..=b,
                )(input)
            }
        }
        map(
            field_line_parser(
                "target area:",
                separated_pair(range_parser("x="), trim(tag(",")), range_parser("y=")),
            ),
            |(range_x, range_y)| Self { range_x, range_y },
        )(input)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 17,
    name: "Trick Shot",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let target_area = TargetArea::from_str(input)?;

            println!("Target area: {:?}", target_area);

            // Process
            Ok(0u64.into())
        },
    ],
};
