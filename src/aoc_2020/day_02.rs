use crate::aoc::prelude::*;
use nom::{
    bytes::complete::{tag, take},
    combinator::{map, rest},
    error::context,
    sequence::separated_pair,
};
use std::convert::TryInto;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
            vec![Unsigned(378), Unsigned(280)],
        "1-3 a: abcde
1-3 b: cdefg
2-9 c: ccccccccc",
        vec![2u64, 1].answer_vec()
    }
}

#[derive(Debug)]
struct PasswordPolicy {
    a: u32,
    b: u32,
    character: char,
}

impl Parseable<'_> for PasswordPolicy {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        use nom::character::complete::u32 as cu32;
        context(
            "password policy",
            map(
                separated_pair(separated_pair(cu32, tag("-"), cu32), tag(" "), take(1usize)),
                |((a, b), s): ((u32, u32), &str)| PasswordPolicy {
                    a,
                    b,
                    character: s.chars().next().unwrap(),
                },
            ),
        )(input.trim())
    }
}

#[derive(Debug)]
struct Password {
    policy: PasswordPolicy,
    password: String,
}

impl Parseable<'_> for Password {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        context(
            "password",
            separated_pair(PasswordPolicy::parser, tag(": "), rest),
        )(input.trim())
        .map(|(next, res)| {
            (
                next,
                Password {
                    policy: res.0,
                    password: res.1.to_string(),
                },
            )
        })
    }
}

impl Password {
    fn valid_part_a(&self) -> bool {
        let char_count = self
            .password
            .matches(self.policy.character)
            .count()
            .try_into()
            .unwrap();
        (self.policy.a..=self.policy.b).contains(&char_count)
    }

    fn valid_part_b(&self) -> bool {
        // Just going to naively assume that the string is long
        // enough to contain both characters
        macro_rules! check {
            ($v:expr) => {
                self.password
                    .chars()
                    .nth(($v - 1).try_into().unwrap())
                    .unwrap()
                    == self.policy.character
            };
        }
        let a = check!(self.policy.a);
        let b = check!(self.policy.b);
        (a || b) && !(a && b)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 2,
    name: "Password Philosophy",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let passwords = Password::gather(input.lines())?;

            // Processing
            Ok(Answer::Unsigned(
                passwords.iter().filter_count(|p| p.valid_part_a()),
            ))
        },
        // Part b)
        |input| {
            // Generation
            let passwords = Password::gather(input.lines())?;

            // Processing
            Ok(Answer::Unsigned(
                passwords.iter().filter_count(|p| p.valid_part_b()),
            ))
        },
    ],
};
