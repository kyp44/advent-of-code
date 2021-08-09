use crate::aoc::prelude::*;
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::{digit1, multispace1},
    combinator::map,
    sequence::tuple,
    Finish,
};
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Number;

    solution_test! {
        vec![Number(542529149), Number(75678618)],
        "previous: 5
35
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
        vec![127, 62].answer_vec()
    }
}

type Number = u64;

enum Validation {
    Valid,
    Invalid(u64),
}

struct XmasPacket {
    previous: usize,
    numbers: Vec<Number>,
}

impl FromStr for XmasPacket {
    type Err = NomParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (input, previous) = map(
            tuple((tag("previous:"), multispace1, digit1, multispace1)),
            |(_, _, ns, _): (&str, &str, &str, &str)| ns.parse().unwrap(),
        )(input)
        .finish()?;
        let numbers = Number::gather(input.lines())?;
        Ok(XmasPacket { previous, numbers })
    }
}

impl XmasPacket {
    fn validate(&self) -> Validation {
        for (i, v) in self.numbers.iter().enumerate().skip(self.previous) {
            // Check that the current value is some sum of the previous numbers
            if self.numbers[i - self.previous..i]
                .iter()
                .combinations(2)
                .all(|vals| vals.into_iter().sum::<u64>() != *v)
            {
                return Validation::Invalid(*v);
            }
        }
        Validation::Valid
    }

    fn exploit(&self, invalid_n: u64) -> Option<u64> {
        // Go through each number and look for the contiguous sequence
        for (ai, a) in self.numbers.iter().enumerate() {
            let mut sum = *a;
            for (bi, b) in self.numbers[ai + 1..].iter().enumerate() {
                sum += *b;
                if sum == invalid_n {
                    let slice = &self.numbers[ai..=ai + bi + 1];
                    return Some(slice.iter().min().unwrap() + slice.iter().max().unwrap());
                }
            }
        }

        None
    }
}

fn verify_invalid(packet: &XmasPacket) -> AocResult<u64> {
    match packet.validate() {
        Validation::Valid => Err(AocError::Process(
            "Packet was unexpectedly valid, guess it can't be exploited!".into(),
        )),
        Validation::Invalid(v) => Ok(v),
    }
}

pub const SOLUTION: Solution = Solution {
    day: 9,
    name: "Encoding Error",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let packet: XmasPacket = input.parse()?;

            // Processing
            verify_invalid(&packet).map(|n| n.into())
        },
        // Part b)
        |input| {
            // Generation
            let packet: XmasPacket = input.parse()?;

            // Processing
            let invalid_n = verify_invalid(&packet)?;
            packet
                .exploit(invalid_n)
                .ok_or_else(|| AocError::Process("Could not exploit packet!".into()))
                .map(|n| n.into())
        },
    ],
};
