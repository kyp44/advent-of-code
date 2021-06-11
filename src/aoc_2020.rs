use super::aoc::{AocError, ParseError, ParseResult};
use std::ops::RangeInclusive;
use nom::{
    Err as NomErr,
    Parser,
    error::context,
    sequence::separated_pair,
    character::complete::digit1,
    bytes::complete::tag,
    bytes::complete::take,
    combinator::rest,
};

#[cfg(test)]
mod tests{
    use super::*;
    
    #[test]
    fn year_2020_day_02() {
        let input = "1-3 a: abcde\
                     1-3 b: cdefg\
                     2-9 c: ccccccccc";
        let input = "asdfasdfasdfsd";
        let result = password_philosophy(input);
        match result {
            Ok(v) => assert_eq!(v, 2),
            Err(e) => panic!("{}", e),
        }
    }
}

type CountRange = RangeInclusive<u32>;

#[derive(Debug)]
struct PasswordPolicy {
    count_range: CountRange,
    character: char,
}

impl PasswordPolicy {
    fn parse(input: &str) -> ParseResult<PasswordPolicy> {
        context(
            "password policy",
            separated_pair(
                separated_pair(digit1, tag("-"), context("fuckestein", digit1)),
                tag(" "),
                take(1usize),
            )
        )(input).map(|(next, res)| {
            // Note that we can unwrap safely here because the range bounds should be digits
            (next, PasswordPolicy{
                count_range: res.0.0.parse().unwrap()..=res.0.1.parse().unwrap(),
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

impl Password {
    fn parse(input: &str) -> ParseResult<Password> {
        context(
            "password",
            separated_pair(PasswordPolicy::parse, tag(": "), rest),
        )(input).map(|(next, res)| {
            (next, Password{
                policy: res.0,
                password: res.1.to_string(),
            })
        })
    }
}

pub fn password_philosophy(input: &str) -> Result<u32, AocError> {
    // Generation
    let passwords: Vec<Password> = input.lines()
        .map(|l| Password::parse(l).map(|t| t.1))
        .collect::<Result<Vec<Password>, NomErr<ParseError>>>()?;

    for p in passwords {
        println!("{:?}", p)
    }
    
    // Solution
    Ok(0)
}
