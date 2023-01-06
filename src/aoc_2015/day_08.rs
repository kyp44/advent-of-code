use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(1333), Unsigned(2046)],
    "\"\"
\"abc\"
\"aaa\\\"aaa\"
\"\\x27\"",
    vec![12u64, 19].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use itertools::{process_results, ProcessResults};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{anychar, none_of},
        combinator::value,
        multi::many0,
        sequence::{delimited, preceded, tuple},
        Finish,
    };

    /// Trait to escape or encode a string.
    trait ProgramString {
        /// Compress the item into an escaped string as it would actually be stored in memory.
        fn escaped(&self) -> AocResult<String>;
        /// Encode the item by escaping special characters.
        fn encoded(&self) -> String;
    }
    impl ProgramString for str {
        fn escaped(&self) -> AocResult<String> {
            delimited(
                tag("\""),
                many0(alt((
                    preceded(
                        tag("\\"),
                        alt((
                            value('\\', tag("\\")),
                            value('"', tag("\"")),
                            value('-', tuple((tag("x"), anychar, anychar))),
                        )),
                    ),
                    none_of("\""),
                ))),
                tag("\""),
            )(self)
            .finish()
            .discard_input()
            .map(|s| s.into_iter().collect())
            .map_err(|e: NomParseError| e.into())
        }

        fn encoded(&self) -> String {
            format!("\"{}\"", self.replace('\\', "\\\\").replace('\"', "\\\""))
        }
    }

    /// Santa's list that can be parsed from text input.
    pub struct List<'a> {
        /// The list of string items.
        items: Vec<&'a str>,
    }
    impl<'a> List<'a> {
        /// Parse the list from text input.
        pub fn from_str(s: &'a str) -> Self {
            List {
                items: s.lines().map(str::trim).collect(),
            }
        }

        /// Find the total size of the string literals for all items
        pub fn literal_size(&self) -> usize {
            self.items.iter().map(|item| item.len()).sum()
        }

        /// Find the total size of the escaped strings in memory.
        pub fn escaped_size(&self) -> AocResult<usize> {
            process_results(
                self.items.iter().map(|item| item.escaped()),
                |iter: ProcessResults<_, AocError>| iter.map(|esc| esc.len()).sum(),
            )
        }

        /// Find the total size of the escaped strings after escaping special characters.
        pub fn encoded_size(&self) -> usize {
            self.items.iter().map(|item| item.encoded().len()).sum()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 8,
    name: "Matchsticks",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let list = List::from_str(input.expect_input()?);

            // Process
            Ok(Answer::Unsigned(
                (list.literal_size() - list.escaped_size()?)
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            // Generation
            let list = List::from_str(input.expect_input()?);

            // Process
            Ok(Answer::Unsigned(
                (list.encoded_size() - list.literal_size())
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
