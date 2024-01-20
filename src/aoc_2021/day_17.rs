use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "target area: x=20..30, y=-10..-5";
            answers = unsigned![45, 112];
        }
        actual_answers = unsigned![3003, 940];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::{field_line_parser, trim};
    use cgmath::{Point2, Vector2};
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
    impl Parsable<'_> for TargetArea {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            /// This is a [`nom`] parser that parses a single inclusive range.
            ///
            /// This is an internal function of [`TargetArea::parser`].
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
        /// Returns an [`Iterator`] over the peak `y` positions of the probe for each trajectory
        /// for which the probe hits the target.
        pub fn peaks(&self) -> impl Iterator<Item = i32> + '_ {
            // Go through every initial velocity that has a chance of hitting the target
            iproduct!(
                // These ranges were derived analytically, see the notes.
                1..=*self.range_x.end(),
                *self.range_y.start()..=-(*self.range_y.start() + 1)
            )
            .filter_map(|velocity| {
                for point in Trajectory::new(velocity.into()) {
                    if self.range_x.contains(&point.x) && self.range_y.contains(&point.y) {
                        // This is the peak value, see equation (29) in the notes writeup
                        return Some(if velocity.1 <= 0 {
                            0
                        } else {
                            (velocity.1 * (velocity.1 + 1)) / 2
                        });
                    } else if point.y < *self.range_y.start() {
                        return None;
                    }
                }
                None
            })
        }
    }

    /// Position of the probe relative to the submarine.
    type Position = Point2<i32>;

    /// A relative velocity vector.
    type Velocity = Vector2<i32>;

    /// A trajectory taken by the probe, which is an [`Iterator`] over the probe
    /// locations at every step.
    #[derive(new)]
    struct Trajectory {
        /// The current position of the probe.
        #[new(value = "Position::origin()")]
        point: Position,
        /// The current velocity of the probe.
        velocity: Velocity,
    }
    impl Iterator for Trajectory {
        type Item = Position;

        fn next(&mut self) -> Option<Self::Item> {
            let ret = self.point;

            // Determine next point and velocity
            self.point += self.velocity;
            self.velocity += Velocity::new(
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
