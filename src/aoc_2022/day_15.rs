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

10

20";
            answers = unsigned![26, 56000011];
        }
        actual_answers = unsigned![5100463, 11557863040754];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::{collections::HashSet, str::FromStr};

    use super::*;
    use aoc::parse::trim;
    use derive_more::Deref;
    use euclid::Point2D;
    use gcollections::ops::{Bounded, Cardinality, Difference, Empty, IsEmpty};
    use interval::{Interval, IntervalSet, ops::Range};
    use nom::{
        bytes::complete::tag,
        combinator::map,
        sequence::{preceded, separated_pair},
    };

    /// A 2D point that can be parsed from the input text.
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
                |t| Self(Point2D::from(t).to_isize()),
            )
            .parse(input)
        }
    }

    /// A report for a single sensor.
    #[derive(Debug)]
    struct SensorReport {
        /// The location of the sensor.
        sensor: AnyGridPoint,
        /// The location of the nearest beacon.
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
            )
            .parse(input)
        }
    }
    impl SensorReport {
        /// Returns the interval that the sensor covers for a given row, which may be empty.
        ///
        /// This is based on the (Manhattan) distance of the sensor to the nearest beacon.
        pub fn coverage_row(&self, row: isize) -> Interval<isize> {
            let distance = (self.nearest_beacon - self.sensor).manhattan_len();

            let dy = (self.sensor.y - row).abs();
            if dy > distance {
                Interval::empty()
            } else {
                let dx = distance - dy;
                Interval::new(self.sensor.x - dx, self.sensor.x + dx)
            }
        }
    }

    /// A collection of sensor reports.
    #[derive(Debug)]
    pub struct SensorReports {
        /// The sensor reports.
        reports: Vec<SensorReport>,
        /// The row for which to determine the number of positions where a beacon
        /// cannot be (part one).
        row: isize,
        /// The `x` and `y` limits when looking for the distress beacon (part two).
        limit: isize,
    }
    impl FromStr for SensorReports {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let secs = s.sections(3)?;

            /// Attempts to parse a section of the input as a number.
            ///
            /// This is an internal function of [`SensorReport::from_str`].
            fn parse_number(name: &str, input: &str) -> AocResult<isize> {
                isize::from_str(input.trim()).map_err(|_| {
                    AocError::InvalidInput(format!("The {name} section is not a number!").into())
                })
            }

            Ok(Self {
                reports: SensorReport::gather(secs[0].lines())?,
                row: parse_number("part one row", secs[1])?,
                limit: parse_number("part two x/y limit", secs[2])?,
            })
        }
    }
    impl SensorReports {
        /// Returns an interval set that includes the coverage for all sensors for a
        /// particular `row`, which could be empty.
        fn row_coverage(&self, row: isize) -> IntervalSet<isize> {
            // Build the intervals on the row
            let mut row_ints = IntervalSet::empty();
            row_ints.extend(self.reports.iter().filter_map(|sr| {
                let int = sr.coverage_row(row);
                if int.is_empty() { None } else { Some(int) }
            }));

            row_ints
        }

        /// Returns the number of positions where the beacon cannot be in the row
        /// provided as part of the input (part one).
        pub fn row_no_beacon_positions(&self) -> u64 {
            let row_ints = self.row_coverage(self.row);

            // Unique beacons on the row
            let beacons = self
                .reports
                .iter()
                .filter_map(|sr| (sr.nearest_beacon.y == self.row).then_some(sr.nearest_beacon))
                .collect::<HashSet<_>>();

            (row_ints.size() - beacons.len()).try_into().unwrap()
        }

        /// Locates the distress beacon position, searching within the limited space
        /// provided as part of the input.
        ///
        /// Returns the tuning frequency based on the distress beacon coordinates
        /// (part two).
        pub fn distress_beacon_tuning_frequency(&self) -> AocResult<u64> {
            let mut whole_row = IntervalSet::empty();
            whole_row.extend_one(Interval::new(0, self.limit));

            // Go through every row looking for a gap in coverage
            for row in 0..=self.limit {
                let positions = whole_row.clone().difference(&self.row_coverage(row));
                if let Some(int) = positions.into_iter().next()
                    && !int.is_empty()
                {
                    // We have a position where the beacon can be!
                    let col = int.lower();

                    return Ok((col * 4000000 + row).try_into().unwrap());
                }
            }
            Err(AocError::NoSolution)
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
            // Process
            Ok(input
                .expect_data::<SensorReports>()?
                .row_no_beacon_positions()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<SensorReports>()?
                .distress_beacon_tuning_frequency()?
                .into())
        },
    ],
};
