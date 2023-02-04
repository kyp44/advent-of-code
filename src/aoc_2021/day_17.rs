use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(3003), Unsigned(940)],
    "target area: x=20..30, y=-10..-5",
    vec![45u64, 112].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::{field_line_parser, trim};
    use cgmath::{Vector2, Zero};
    use derive_new::new;
    use itertools::iproduct;
    use nom::{bytes::complete::tag, combinator::map, sequence::separated_pair};
    use std::ops::RangeInclusive;

    /// The target area for the probe, which can be parsed from text input.
    #[derive(Debug)]
    pub struct TargetArea {
        /// Inclusive range of `x` values included in the target area.
        range_x: RangeInclusive<i32>,
        /// Inclusive range of `y` values included in the target area.
        range_y: RangeInclusive<i32>,
    }
    impl Parseable<'_> for TargetArea {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            /// Sub-function of [TargetArea::parser] that parses a single range.
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
                    separated_pair(
                        range_parser("x="),
                        trim(false, tag(",")),
                        range_parser("y="),
                    ),
                ),
                |(range_x, range_y)| Self { range_x, range_y },
            )(input)
        }
    }
    impl TargetArea {
        /// Returns an [Iterator] over the peak `y` positions of the probe for each trajectory
        /// for which the probe hits the target.
        pub fn peaks(&self) -> impl Iterator<Item = i32> + '_ {
            // TODO: Derive in the notes document.
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

    /// A 2D vector in the coordinate system in which the submarine
    /// is at the origin.
    type Vector = Vector2<i32>;

    /// A trajectory taken by the probe, which is an [Iterator] over the probe
    /// locations at every step.
    #[derive(new)]
    struct Trajectory {
        /// The current position of the probe.
        #[new(value = "Vector::zero()")]
        point: Vector,
        /// The current velocity of the probe.
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
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 17,
    name: "Trick Shot",
    preprocessor: Some(|input| Ok(Box::new(TargetArea::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<TargetArea>()?
                    .peaks()
                    .max()
                    .unwrap()
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<TargetArea>()?
                    .peaks()
                    .count()
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
