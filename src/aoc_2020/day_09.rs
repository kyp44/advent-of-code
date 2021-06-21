use super::super::aoc::{
    AocError,
    Parseable,
    ParseResult,
    Solution,
};
use std::str::FromStr;
use itertools::Itertools;

#[cfg(test)]
mod tests{
    use super::*;
    use crate::solution_test;

    solution_test! {
        "35
20
15
25
47
40
62
55
65
95
102
117
150
182
127
219
299
277
309
576",
        vec![127],
        vec![]
    }
}

type Number = u32;

struct XmasPacket {
    previous: usize;
    numbers: Vec<Numbers>;
}

impl XmasPacket {
    fn new(previous: usize, numbers: Vec<Numbers>) {
        XmasPacket
    }
}

impl FromStr for XmasPacket(input: &str) {
    type Err = AocError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
    }
}

pub const SOLUTION: Solution = Solution {
    day: 9,
    name: "Encoding Error",
    solver: |input| {
        // Generation
        let numbers = Number::gather(input.lines());

        // Processing
        let answers = vec![];

        Ok(answers)
    },
};
