use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::{expensive_test, solution_test};
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(492982), Unsigned(6989950)],
    "1",
    vec![82350u64].answer_vec()
    }

    expensive_test! {
    "1",
    vec![None, Some(Unsigned(1166642))]
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
    impl Parseable<'_> for Sequence {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(digit1, |ds: &str| Sequence {
                current: ds.to_string().into(),
            })(input.trim())
        }
    }
    impl Iterator for Sequence {
        type Item = String;

        fn next(&mut self) -> Option<Self::Item> {
            let next = self.current.take();

            self.current = Takeable::new(
                next.split_runs()
                    .map(|s| format!("{}{}", s.len(), s.chars().next().unwrap()))
                    .collect(),
            );
            Some(next)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 10,
    name: "Elves Look, Elves Say",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            //println!("{}", Sequence::from_str(input.expect_input()?)?.nth(40).unwrap().len());
            Ok(Answer::Unsigned(
                Sequence::from_str(input.expect_input()?)?
                    .nth(40)
                    .unwrap()
                    .len()
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            //println!("{}", Sequence::from_str(input.expect_input()?)?.nth(50).unwrap().len());
            Ok(Answer::Unsigned(
                Sequence::from_str(input.expect_input()?)?
                    .nth(50)
                    .unwrap()
                    .len()
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
