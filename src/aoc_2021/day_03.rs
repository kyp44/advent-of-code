use std::str::FromStr;

use itertools::Itertools;
use nom::{character::complete::one_of, combinator::map, multi::many1};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(112312)],
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
    vec![198u64].answer_vec()
    }
}

struct ReportLine {
    bit_vec: Vec<bool>,
}

impl Parseable<'_> for ReportLine {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(many1(one_of("01")), |v| Self {
            bit_vec: v
                .into_iter()
                .map(|b| if b == '1' { true } else { false })
                .collect(),
        })(input)
    }
}
impl Into<u64> for ReportLine {
    fn into(self) -> u64 {
        self.bit_vec
            .into_iter()
            .fold(0, |a, b| 2 * a + u64::from(b))
    }
}

struct Report {
    lines: Box<[ReportLine]>,
    size: usize,
}
impl FromStr for Report {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = ReportLine::gather(s.lines())?.into_boxed_slice();
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
    fn power_consumption(&self) -> AocResult<u64> {
        let gamma = (0..self.size)
            .map(|n| {
                let n_ones: usize = self.lines.iter().filter_count(|l| l.bit_vec[n]);
                // 2*x > n  <=>  x > n - x
                2 * n_ones > self.lines.len()
            })
            .collect();
        let epsilon = !gamma.clone();
        let x: u64 = &gamma.into();

        println!("gamma: {}, epsilon: {}", gamma, epsilon);

        Ok(0)
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
    ],
};
