use std::{cmp::Ordering, ops::Not, str::FromStr};

use itertools::Itertools;
use nom::{character::complete::one_of, combinator::map, multi::many1};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(3320834), Unsigned(4481199)],
    "00100
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
01010",
    vec![198u64, 230].answer_vec()
    }
}

#[derive(Clone)]
struct ReportLine {
    bit_vec: Vec<bool>,
}
impl ReportLine {
    fn value(&self) -> u64 {
        self.bit_vec.iter().fold(0, |a, b| 2 * a + u64::from(*b))
    }
}
impl Parseable<'_> for ReportLine {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        map(many1(one_of("01")), |v| Self {
            bit_vec: v.into_iter().map(|b| b == '1').collect(),
        })(input)
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

#[derive(Clone)]
struct Report {
    lines: Vec<ReportLine>,
    size: usize,
}
enum MostCommon {
    Ones,
    Zeros,
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

        Ok(Self { lines, size })
    }
}
impl Report {
    fn most_common(&self, bit_num: usize) -> MostCommon {
        let n_ones: usize = self.lines.iter().filter_count(|l| l.bit_vec[bit_num]);
        match (2 * n_ones).cmp(&self.lines.len()) {
            Ordering::Less => MostCommon::Zeros,
            Ordering::Equal => MostCommon::Equal,
            Ordering::Greater => MostCommon::Ones,
        }
    }

    fn filter_bit(&mut self, bit_num: usize, bit_value: bool) {
        self.lines.retain(|l| l.bit_vec[bit_num] == bit_value)
    }

    fn power_consumption(&self) -> AocResult<u64> {
        let gamma: ReportLine = (0..self.size)
            .map(|n| matches!(self.most_common(n), MostCommon::Ones))
            .collect();
        let gamma_val = gamma.value();
        let epsilon_val = (!gamma).value();

        Ok(gamma_val * epsilon_val)
    }

    fn calculate_rating<F: Fn(MostCommon) -> bool>(&self, criteria: F) -> AocResult<u64> {
        let mut report = self.clone();

        for n in 0..self.size {
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

    fn life_support_rating(&self) -> AocResult<u64> {
        let oxygen_rating = self.calculate_rating(|com| !matches!(com, MostCommon::Zeros))?;
        let co2_rating = self.calculate_rating(|com| matches!(com, MostCommon::Zeros))?;

        Ok(oxygen_rating * co2_rating)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 3,
    name: "Binary Diagnostic",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let report = Report::from_str(input)?;

            // Process
            Ok(report.power_consumption()?.into())
        },
        // Part b)
        |input| {
            // Generation
            let report = Report::from_str(input)?;

            // Process
            Ok(report.life_support_rating()?.into())
        },
    ],
};
