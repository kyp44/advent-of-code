use cgmath::{Vector2, Zero};
use itertools::iproduct;
use nom::{bytes::complete::tag, combinator::map, sequence::separated_pair};
use std::ops::RangeInclusive;

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
    vec![Unsigned(3003), Unsigned(940)],
    "target area: x=20..30, y=-10..-5",
    vec![45u64, 112].answer_vec()
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
    /// Trajectory peaks for tractories that hit the target
    fn peaks(&self) -> impl Iterator<Item = i32> + '_ {
        // These were derived analytically
        let vyi_max = -(*self.range_y.start() + 1);
        let vxi_min =
            ((f32::sqrt((1 + *self.range_x.start() * 8) as f32) - 1f32) / 2f32).ceil() as i32;

        // Go through every initial velocity that has a chance of hitting the target
        iproduct!(
            vxi_min..=*self.range_x.end(),
            *self.range_y.start()..=vyi_max
        )
        .filter_map(|velocity| {
            let mut peak = 0;
            for point in Trajectory::new(velocity.into()) {
                peak = peak.max(point.y);

                if self.range_x.contains(&point.x) && self.range_y.contains(&point.y) {
                    return Some(peak);
                } else if point.y < *self.range_y.start() {
                    return None;
                }
            }
            None
        })
    }
}

type Vector = Vector2<i32>;

#[derive(new)]
struct Trajectory {
    #[new(value = "Vector::zero()")]
    point: Vector,
    velocity: Vector,
}
impl Iterator for Trajectory {
    type Item = Vector;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.point;

        // Determine next point and velocity
        self.point += self.velocity;
        self.velocity += Vector::new(
            match self.velocity.x.cmp(&0) {
                std::cmp::Ordering::Less => 1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => -1,
            },
            -1,
        );

        Some(ret)
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

            // Process
            Ok(Answer::Unsigned(
                target_area.peaks().max().unwrap().try_into().unwrap(),
            ))
        },
        // Part b)
        |input| {
            // Generation
            let target_area = TargetArea::from_str(input)?;

            // Process
            Ok(Answer::Unsigned(
                target_area.peaks().count().try_into().unwrap(),
            ))
        },
    ],
};
