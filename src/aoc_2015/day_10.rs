use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "1";
            answers = unsigned![82350];
        }
        expensive_example {
            input = "1";
            answers = &[None, Some(Answer::Unsigned(1166642))];
        }
        actual_answers = unsigned![492982, 6989950];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use nom::{character::complete::digit1, combinator::map};
    use takeable::Takeable;

    /// Number sequence that can be parsed from text input.
    ///
    /// Also an [`Iterator`] to iterate over the look-and-say sequence of sequences.
    pub struct Sequence {
        /// Sequence for the next iteration.
        current: Takeable<String>,
    }
    impl Parsable<'_> for Sequence {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(digit1, |ds: &str| Sequence {
                current: ds.to_string().into(),
            })
            .parse(input.trim())
        }
    }
    impl Iterator for Sequence {
        type Item = String;

        fn next(&mut self) -> Option<Self::Item> {
            use std::fmt::Write;
            let next = self.current.take();

            self.current = Takeable::new(next.split_runs().fold(String::new(), |mut out, s| {
                let _ = write!(out, "{}{}", s.len(), s.chars().next().unwrap());
                out
            }));
            Some(next)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 10,
    name: "Elves Look, Elves Say",
    // NOTE: Sequence is an iterator so needs mutated, so we just parse it in each part.
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            //println!("{}", Sequence::from_str(input.expect_text()?)?.nth(40).unwrap().len());
            Ok(Answer::Unsigned(
                Sequence::from_str(input.expect_text()?)?
                    .nth(40)
                    .unwrap()
                    .len()
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            //println!("{}", Sequence::from_str(input.expect_text()?)?.nth(50).unwrap().len());
            Ok(Answer::Unsigned(
                Sequence::from_str(input.expect_text()?)?
                    .nth(50)
                    .unwrap()
                    .len()
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
