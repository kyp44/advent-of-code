use std::str::FromStr;

use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "Sensor at x=2, y=18: closest beacon is at x=-2, y=15
Sensor at x=9, y=16: closest beacon is at x=10, y=16
Sensor at x=13, y=2: closest beacon is at x=15, y=3
Sensor at x=12, y=14: closest beacon is at x=10, y=16
Sensor at x=10, y=20: closest beacon is at x=10, y=16
Sensor at x=14, y=17: closest beacon is at x=10, y=16
Sensor at x=8, y=7: closest beacon is at x=2, y=10
Sensor at x=2, y=0: closest beacon is at x=2, y=10
Sensor at x=0, y=11: closest beacon is at x=2, y=10
Sensor at x=20, y=14: closest beacon is at x=25, y=17
Sensor at x=17, y=20: closest beacon is at x=21, y=22
Sensor at x=16, y=7: closest beacon is at x=15, y=3
Sensor at x=14, y=3: closest beacon is at x=15, y=3
Sensor at x=20, y=1: closest beacon is at x=15, y=3

10";
            answers = unsigned![123];
        }
        actual_answers = unsigned![123];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::str::FromStr;

    use super::*;
    use aoc::parse::trim;
    use cgmath::Point2;
    use derive_more::Deref;
    use nom::{
        bytes::complete::tag,
        combinator::map,
        sequence::{preceded, separated_pair},
    };

    #[derive(Deref)]
    struct ParsePoint(AnyGridPoint);
    impl Parsable<'_> for ParsePoint {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(
                    preceded(tag("x="), nom::character::complete::i32),
                    trim(false, tag(",")),
                    preceded(tag("y="), nom::character::complete::i32),
                ),
                |(x, y)| Self(Point2::new(x, y).try_point_into().unwrap()),
            )(input)
        }
    }

    #[derive(Debug)]
    struct SensorReport {
        sensor: AnyGridPoint,
        nearest_beacon: AnyGridPoint,
    }
    impl Parsable<'_> for SensorReport {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(
                    preceded(tag("Sensor at "), ParsePoint::parser),
                    tag(": closest beacon is at "),
                    ParsePoint::parser,
                ),
                |(s, b)| Self {
                    sensor: *s,
                    nearest_beacon: *b,
                },
            )(input)
        }
    }

    #[derive(Debug)]
    pub struct SensorReports {
        reports: Vec<SensorReport>,
        row: isize,
    }
    impl FromStr for SensorReports {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let secs = s.sections(2)?;
            Ok(Self {
                reports: SensorReport::gather(secs[0].lines())?,
                row: isize::from_str(secs[1]).map_err(|_| {
                    AocError::InvalidInput("The second section is not a number!".into())
                })?,
            })
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 15,
    name: "Beacon Exclusion Zone",
    preprocessor: Some(|input| Ok(Box::new(SensorReports::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Generation
            println!("TODO: {:?}", input.expect_data::<SensorReports>()?);

            // Process
            Ok(0u64.into())
        },
    ],
};
