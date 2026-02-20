use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "00100
11110
10110
10111
10101
01111
00111
11100
10000
11001
00010
01010";
            answers = unsigned![198, 230];
        }
        actual_answers = unsigned![3320834, 4481199];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use itertools::Itertools;
    use nom::{character::complete::one_of, combinator::map, multi::many1};
    use std::{cmp::Ordering, ops::Not};

    /// A single line in the diagnostic report, which can be parsed from
    /// text input.
    #[derive(Clone)]
    struct ReportLine {
        /// Ordered list of the bit values from most significant to
        /// least significant.
        bit_vec: Vec<bool>,
    }
    impl ReportLine {
        /// Returns the decimal value of the bits.
        fn value(&self) -> u64 {
            self.bit_vec.iter().fold(0, |a, b| 2 * a + u64::from(*b))
        }
    }
    impl Parsable for ReportLine {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(many1(one_of("01")), |v| Self {
                bit_vec: v.into_iter().map(|b| b == '1').collect(),
            })
            .parse(input)
        }
    }
    impl FromIterator<bool> for ReportLine {
        fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
            Self {
                bit_vec: iter.into_iter().collect(),
            }
        }
    }
    impl Not for ReportLine {
        type Output = ReportLine;

        fn not(mut self) -> Self::Output {
            self.bit_vec.iter_mut().for_each(|b| *b = !*b);
            self
        }
    }

    /// The full diagnostic report, which can be parsed from
    /// text input.
    #[derive(Clone)]
    pub struct Report {
        /// The ordered list of report lines.
        lines: Vec<ReportLine>,
        /// The number of bits in each line.
        bit_depth: usize,
    }
    /// The result from assessing the most common value from
    /// a collection of bits.
    enum MostCommon {
        /// Zero bits are most common.
        Zeros,
        /// One bits are most common.
        Ones,
        /// There are equal zero and one bits.
        Equal,
    }
    impl FromStr for Report {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let lines = ReportLine::gather(s.lines())?;
            let size = lines[0].bit_vec.len();

            if !lines.iter().map(|l| l.bit_vec.len()).all_equal() {
                return Err(AocError::InvalidInput(
                    "Not all report lines have the same size".into(),
                ));
            }

            Ok(Self {
                lines,
                bit_depth: size,
            })
        }
    }
    impl Report {
        /// Determines which is the most common bit, if any, given a bit number
        /// to examine in each report line.
        ///
        /// Will panic if the bit number is invalid.
        fn most_common(&self, bit_num: usize) -> MostCommon {
            let n_ones: usize = self.lines.iter().filter_count(|l| l.bit_vec[bit_num]);
            match (2 * n_ones).cmp(&self.lines.len()) {
                Ordering::Less => MostCommon::Zeros,
                Ordering::Equal => MostCommon::Equal,
                Ordering::Greater => MostCommon::Ones,
            }
        }

        /// Keeps only the report lines where the specified `bit_number` matches
        /// the specified `bit_value`.
        fn filter_bit(&mut self, bit_num: usize, bit_value: bool) {
            self.lines.retain(|l| l.bit_vec[bit_num] == bit_value)
        }

        /// Calculates the power consumption by first determining the gamma
        /// and epsilon rates.
        pub fn power_consumption(&self) -> AocResult<u64> {
            let gamma: ReportLine = (0..self.bit_depth)
                .map(|n| matches!(self.most_common(n), MostCommon::Ones))
                .collect();
            let gamma_val = gamma.value();
            let epsilon_val = (!gamma).value();

            Ok(gamma_val * epsilon_val)
        }

        /// Calculates a rating by filtering the report lines according to the
        /// most or least common value for each bit number.
        ///
        /// The `criteria` closure should take a [`MostCommon`] value for the
        /// bit number and return what the bit value should be for that
        /// bit number in the lines that are kept.
        fn calculate_rating(&self, criteria: impl Fn(MostCommon) -> bool) -> AocResult<u64> {
            let mut report = self.clone();

            for n in 0..self.bit_depth {
                if report.lines.len() == 1 {
                    break;
                }

                report.filter_bit(n, criteria(report.most_common(n)))
            }

            if report.lines.len() == 1 {
                Ok(report.lines[0].value())
            } else {
                Err(AocError::Process(
                    "Rating did not filter report lines down to only one".into(),
                ))
            }
        }

        /// Determines the life support rating by first filtering the report
        /// lines to find the oxygen generator and CO2 scrubber ratings.
        pub fn life_support_rating(&self) -> AocResult<u64> {
            let oxygen_rating = self.calculate_rating(|com| !matches!(com, MostCommon::Zeros))?;
            let co2_rating = self.calculate_rating(|com| matches!(com, MostCommon::Zeros))?;

            Ok(oxygen_rating * co2_rating)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 3,
    name: "Binary Diagnostic",
    preprocessor: Some(|input| Ok(Box::new(Report::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<Report>()?.power_consumption()?.into())
        },
        // Part two
        |input| {
            // Process
            Ok(input.expect_data::<Report>()?.life_support_rating()?.into())
        },
    ],
};
