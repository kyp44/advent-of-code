use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "previous: 5
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
576";
            answers = unsigned![127, 62];
        }
        actual_answers = unsigned![542529149, 75678618];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use itertools::Itertools;
    use nom::{
        Finish,
        bytes::complete::tag,
        character::complete::multispace1,
        combinator::map,
        sequence::{delimited, preceded},
    };
    use std::{convert::TryInto, str::FromStr};

    /// Type to use for numbers in a packet.
    type Number = u64;

    /// Packet validation result.
    enum Validation {
        /// The packet is valid.
        Valid,
        /// The packet is invalid with the first invalid number.
        Invalid(Number),
    }

    /// A packet of data, which can be parsed from text input.
    pub struct XmasPacket {
        /// Length of the preamble.
        preamble_len: usize,
        /// The list of numbers.
        numbers: Vec<Number>,
    }
    impl FromStr for XmasPacket {
        type Err = NomParseError;
        fn from_str(input: &str) -> Result<Self, Self::Err> {
            let (input, previous) = map(
                preceded(
                    tag("previous:"),
                    delimited(multispace1, nom::character::complete::u64, multispace1),
                ),
                |n| n.try_into().unwrap(),
            )
            .parse(input)
            .finish()?;
            let numbers = Number::gather(input.lines())?;
            Ok(XmasPacket {
                preamble_len: previous,
                numbers,
            })
        }
    }

    impl XmasPacket {
        /// Validates the packet and returns the status.
        fn validate(&self) -> Validation {
            for (i, v) in self.numbers.iter().enumerate().skip(self.preamble_len) {
                // Check that the current value is some sum of the previous numbers
                if self.numbers[i - self.preamble_len..i]
                    .iter()
                    .combinations(2)
                    .all(|values| values.into_iter().sum::<u64>() != *v)
                {
                    return Validation::Invalid(*v);
                }
            }
            Validation::Valid
        }

        /// Exploits the encryption by finding at least two contiguous numbers that sum to the number passed.
        ///
        /// If such a contiguous set is found, returns the sum of the smallest and largest of these.
        pub fn exploit(&self, invalid_n: Number) -> Option<Number> {
            // Go through each number and look for the contiguous sequence
            for (ai, a) in self.numbers.iter().enumerate() {
                let mut sum = *a;
                for (bi, b) in self.numbers[ai + 1..].iter().enumerate() {
                    sum += *b;
                    if sum == invalid_n {
                        let slice = &self.numbers[ai..=ai + bi + 1];
                        let range = slice.iter().copied().range().unwrap();
                        return Some(range.start() + range.end());
                    }
                }
            }

            None
        }

        /// Validates the packet and returns the answer if invalid or an error if valid.
        pub fn validate_answer(&self) -> AocResult<Number> {
            match self.validate() {
                Validation::Valid => Err(AocError::Process(
                    "Packet was unexpectedly valid, guess it can't be exploited!".into(),
                )),
                Validation::Invalid(v) => Ok(v),
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 9,
    name: "Encoding Error",
    preprocessor: Some(|input| Ok(Box::new(input.parse::<XmasPacket>()?).into())),
    solvers: &[
        // Part one
        |input| {
            // Processing
            input
                .expect_data::<XmasPacket>()?
                .validate_answer()
                .map(|n| n.into())
        },
        // Part two
        |input| {
            // Processing
            let packet = input.expect_data::<XmasPacket>()?;
            packet
                .exploit(packet.validate_answer()?)
                .ok_or_else(|| AocError::Process("Could not exploit packet!".into()))
                .map(|n| n.into())
        },
    ],
};
