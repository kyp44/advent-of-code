use super::super::aoc::{FilterCount, ParseResult, Parseable, Solution};
use nom::{
    bytes::complete::{tag, take},
    character::complete::digit1,
    combinator::{map, rest},
    error::context,
    sequence::separated_pair,
};
use std::convert::TryInto;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
            vec![378, 280],
        "1-3 a: abcde
1-3 b: cdefg
2-9 c: ccccccccc",
        vec![Some(2), Some(1)]
    }
}

#[derive(Debug)]
struct PasswordPolicy {
    a: u32,
    b: u32,
    character: char,
}

impl Parseable for PasswordPolicy {
    fn parser(input: &str) -> ParseResult<Self> {
        context(
            "password policy",
            map(
                separated_pair(
                    separated_pair(digit1, tag("-"), digit1),
                    tag(" "),
                    take(1usize),
                ),
                |res: ((&str, &str), &str)| PasswordPolicy {
                    a: res.0 .0.parse().unwrap(),
                    b: res.0 .1.parse().unwrap(),
                    character: res.1.chars().next().unwrap(),
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

impl Parseable for Password {
    fn parser(input: &str) -> ParseResult<Self> {
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
                    == self.policy.character;
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
            Ok(passwords.iter().filter_count(|p| p.valid_part_a()))
        },
        // Part b)
        |input| {
            // Generation
            let passwords = Password::gather(input.lines())?;

            // Processing
            Ok(passwords.iter().filter_count(|p| p.valid_part_b()))
        },
    ],
};
