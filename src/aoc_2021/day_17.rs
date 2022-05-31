use std::ops::RangeInclusive;

use fraction::{Fraction, GenericFraction};
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
impl TargetArea {
    fn highest_peak(&self) -> u64 {
        let range_x: RangeInclusive<u64> =
            (*self.range_x.start()).try_into().unwrap()..=(*self.range_x.end()).try_into().unwrap();
        let mut biggest_peak = 0;
        // Try different upward initial velocities (the upper bound is arbitrary)
        for vyi in 9..=9 {
            let mut y_peak = 0;
            // Find y step points until the target is hit or it goes below it
            for n in 0.. {
                let y = n * vyi - n * (n - 1) / 2;
                y_peak = y_peak.max(y);
                println!("TODO wtf {} {} {}", y, y_peak, self.range_y.contains(&y));
                if self.range_y.contains(&y) {
                    // In the target range in y, so now see if there is an initial x velocity that puts it in the target area at this step
                    let n: u64 = n.try_into().unwrap();

                    // This was derived analytically such that there is such an initial x velocity iff a <= n <= b
                    let a = Fraction::new(*range_x.start(), n) + Fraction::new(n + 1, 2u64);
                    let b = Fraction::new(*range_x.end(), n) + Fraction::new(n - 1, 2u64);
                    println!("TODO a b {:.3} {:.3}", a, b);
                    if a.ceil() <= b.floor() {
                        // There is such a n initial x velocity
                        biggest_peak = biggest_peak.max(y_peak);
                        println!("TODO have a good one with peak {} at vyi {}!", y_peak, vyi);
                        break;
                    }
                } else if y < *self.range_y.start() {
                    break;
                }
            }
        }
        0
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
            target_area.highest_peak();

            // Process
            Ok(0u64.into())
        },
    ],
};
