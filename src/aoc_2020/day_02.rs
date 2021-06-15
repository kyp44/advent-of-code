use super::super::aoc::{
    ParseResult,
    Parseable,
    Solution,
    CountFilter
};
use nom::{
    bytes::complete::{tag, take},
    character::complete::digit1,
    combinator::rest,
    error::context,
    sequence::separated_pair,
};

#[cfg(test)]
mod tests{
    use super::*;
    use crate::solution_test;

    solution_test! {
        "1-3 a: abcde
1-3 b: cdefg
2-9 c: ccccccccc",
        vec![2, 1],
        vec![378, 280]
    }
}

#[derive(Debug)]
struct PasswordPolicy {
    a: u32,
    b: u32,
    character: char,
}

impl Parseable for PasswordPolicy {
    fn parse(input: &str) -> ParseResult<Self> {
        context(
            "password policy",
            separated_pair(
                separated_pair(digit1, tag("-"), digit1),
                tag(" "),
                take(1usize),
            )
        )(input).map(|(next, res)| {
            // Note that we can unwrap safely here because the range bounds should be digits
            (next, PasswordPolicy{
                a: res.0.0.parse().unwrap(),
                b: res.0.1.parse().unwrap(),
                character: res.1.chars().next().unwrap(),
            })
        })
    }
}

#[derive(Debug)]
struct Password {
    policy: PasswordPolicy,
    password: String,
}

impl Parseable for Password {
    fn parse(input: &str) -> ParseResult<Self> {
        context(
            "password",
            separated_pair(PasswordPolicy::parse, tag(": "), rest),
        )(input.trim()).map(|(next, res)| {
            (next, Password{
                policy: res.0,
                password: res.1.to_string(),
            })
        })
    }
}

impl Password {
    fn valid_part_a(&self) -> bool {
        let char_count = self.password.matches(self.policy.character).count() as u32;
        (self.policy.a..=self.policy.b).contains(&char_count)
    }

    fn valid_part_b(&self) -> bool {
        // Just going to naively assume that the string is long
        // enough to contain both characters
        macro_rules! check {
            ($v:expr) => {
                self.password.chars().nth(($v - 1) as usize).unwrap() == self.policy.character;
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
    solver: |input| {
        // Generation
        let passwords = Password::gather(input.lines())?;

        // Processing
        let answers = vec![
            passwords.iter().filter_count(|p| p.valid_part_a()),
            passwords.iter().filter_count(|p| p.valid_part_b()),
        ];

        Ok(answers)
    }
};
