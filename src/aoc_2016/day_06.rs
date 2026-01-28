use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "eedadn
drvtee
eandsr
raavrd
atevrs
tsrnev
sdttsa
rasrtv
nssdts
ntnada
svetve
tesnvt
vntsnd
vrdear
dvrsen
enarar";
            answers = string!["easter", "advent"];
        }
        actual_answers = string!["qqqluigu", "lsoypmia"];
    }
}

/// Contains solution implementation items.
mod solution {
    use multiset::HashMultiSet;

    use super::*;

    /// Trait for a correcting code.
    pub trait Code {
        /// Returns the corrected character given the multi set of characters
        /// among in a particular position among all received messages.
        fn message_char(char_frequencies: &HashMultiSet<char>) -> char;
    }

    /// The repetition code in part one, which selects the _most_ common
    /// character.
    pub enum RepetitionCode {}
    impl Code for RepetitionCode {
        fn message_char(char_frequencies: &HashMultiSet<char>) -> char {
            *char_frequencies
                .distinct_elements()
                .max_by_key(|c| char_frequencies.count_of(c))
                .unwrap()
        }
    }

    /// The modified repetition code in part two, which selects the _least_
    /// common character.
    pub enum ModifiedRepetitionCode {}
    impl Code for ModifiedRepetitionCode {
        fn message_char(char_frequencies: &HashMultiSet<char>) -> char {
            *char_frequencies
                .distinct_elements()
                .min_by_key(|c| char_frequencies.count_of(c))
                .unwrap()
        }
    }

    /// A transmission consisting of many messages of the same length.
    ///
    /// Can be parsed from text input.
    pub struct Transmission {
        /// The messages in the transmission, guaranteed to be the same
        /// length when parsed from text.
        messages: Vec<String>,
    }
    impl FromStr for Transmission {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut len = None;

            Ok(Self {
                messages: s
                    .lines()
                    .map(|mut msg| {
                        msg = msg.trim();
                        match len {
                            Some(n) => (msg.len() == n).ok_or(AocError::InvalidInput(
                                format!(
                                    "The message '{msg}' does not have the required length of {n}"
                                )
                                .into(),
                            ))?,
                            None => {
                                len = Some(msg.len());
                            }
                        }
                        Ok(String::from(msg))
                    })
                    .collect::<AocResult<Vec<_>>>()?,
            })
        }
    }
    impl Transmission {
        /// Returns the length of all the messages.
        fn message_len(&self) -> usize {
            self.messages[0].len()
        }

        /// Returns the corrected message for a particular [`Code`].
        pub fn corrected_message<C: Code>(&self) -> String {
            let mut msg_iters: Vec<_> = self.messages.iter().map(|m| m.chars()).collect();

            (0..self.message_len())
                .map(|_| {
                    C::message_char(&HashMultiSet::from_iter(
                        msg_iters.iter_mut().map(|mi| mi.next().unwrap()),
                    ))
                })
                .collect()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 6,
    name: "Signals and Noise",
    preprocessor: Some(|input| Ok(Box::new(Transmission::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Transmission>()?
                .corrected_message::<RepetitionCode>()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Transmission>()?
                .corrected_message::<ModifiedRepetitionCode>()
                .into())
        },
    ],
};
