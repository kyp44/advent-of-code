use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "1-3 a: abcde
1-3 b: cdefg
2-9 c: ccccccccc";
            answers = vec![2u64, 1].answer_vec();
        }
        actual_answers = vec![Unsigned(378), Unsigned(280)];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use nom::{
        bytes::complete::tag,
        character::complete::anychar,
        combinator::{map, rest},
        error::context,
        sequence::separated_pair,
    };
    use std::{convert::TryInto, ops::RangeInclusive};

    /// General password policy, which can be parsed from text input.
    pub trait PasswordPolicy: Sized {
        /// Creates the policy from the parameters.
        fn new(a: u32, b: u32, character: char) -> Self;
        /// Validates a string according to the policy.
        fn validate(&self, password: &str) -> bool;
        /// This is a [`nom`] parser.
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            use nom::character::complete::u32 as cu32;
            context(
                "password policy",
                map(
                    separated_pair(separated_pair(cu32, tag("-"), cu32), tag(" "), anychar),
                    |((a, b), s): ((u32, u32), char)| Self::new(a, b, s),
                ),
            )(input.trim())
        }
    }

    /// Behavior for part one.
    pub struct PartOnePolicy {
        /// Range of occurrences of the character in a valid password.
        occurrence_range: RangeInclusive<u32>,
        /// Character that must have a certain number of occurrences.
        character: char,
    }
    impl PasswordPolicy for PartOnePolicy {
        fn new(a: u32, b: u32, character: char) -> Self {
            Self {
                occurrence_range: a..=b,
                character,
            }
        }

        fn validate(&self, password: &str) -> bool {
            let char_count = password.matches(self.character).count().try_into().unwrap();
            self.occurrence_range.contains(&char_count)
        }
    }

    /// Behavior for part two.
    pub struct PartTwoPolicy {
        /// Positions.
        positions: [usize; 2],
        /// Required character.
        character: char,
    }
    impl PasswordPolicy for PartTwoPolicy {
        fn new(a: u32, b: u32, character: char) -> Self {
            Self {
                positions: [a.try_into().unwrap(), b.try_into().unwrap()],
                character,
            }
        }

        fn validate(&self, password: &str) -> bool {
            // Just going to naively assume that the string is long
            // enough to contain both characters.
            let check =
                |position: usize| password.chars().iterations(position).unwrap() == self.character;

            let a = check(self.positions[0]);
            let b = check(self.positions[1]);
            (a || b) && !(a && b)
        }
    }

    /// Full password, including the applicable policy.
    ///
    /// This can be parsed from text input.
    pub struct Password<'a, P: PasswordPolicy> {
        /// The policy for this password.
        policy: P,
        /// The actual password.
        password: &'a str,
    }
    impl<'a, P: PasswordPolicy> Parseable<'a> for Password<'a, P> {
        fn parser(input: &'a str) -> NomParseResult<&str, Self> {
            context("password", separated_pair(P::parser, tag(": "), rest))(input.trim()).map(
                |(next, res)| {
                    (
                        next,
                        Password {
                            policy: res.0,
                            password: res.1,
                        },
                    )
                },
            )
        }
    }
    impl<P: PasswordPolicy> Password<'_, P> {
        /// Validates the password.
        pub fn validate(&self) -> bool {
            self.policy.validate(self.password)
        }
    }

    /// Solves a part of the problem by reading in policies and passwords and counting those that are valid.
    pub fn solve<P: PasswordPolicy>(input: &SolverInput) -> AocResult<Answer> {
        // Generation
        let passwords = Password::<P>::gather(input.expect_input()?.lines())?;

        // Processing
        Ok(Answer::Unsigned(
            passwords.iter().filter_count(|p| p.validate()),
        ))
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 2,
    name: "Password Philosophy",
    preprocessor: None,
    solvers: &[solve::<PartOnePolicy>, solve::<PartTwoPolicy>],
};
