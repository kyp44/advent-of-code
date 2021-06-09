#[path = "aoc.rs"]
mod aoc;

use aoc::{ParseProblem, AocError};
use std::ops::RangeInclusive;
use std::str::FromStr;

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

struct PasswordPolicy {
    count_range: RangeInclusive<u32>,
    character: char,
}

impl FromStr for PasswordPolicy {
    type Err = AocError;
        
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let mut split = s.split_whitespace();
        let err = ParseProblem::new("password policy", s.to_string());
        // TODO
        // 5-11 t: glhbttzvzttkdx
        let range = {
            let range_str = split.next().unwrap();
            let mut split = range_str.split("-");
            let a: u32 = split.next().unwrap().parse().map_err(|e| err)?;
            let b: u32 = split.next()
                .ok_or(ParseProblem::new("range", range_str.to_string()))?
                .parse().map_err(|e| err)?;
        };

        let a = 8;
                
        Ok(PasswordPolicy{ count_range: 0..=10, character: 'a' })
    }
}

struct Password {
    policy: PasswordPolicy,
    password: String,
}

impl FromStr for Password {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let mut split = s.split(": ");

        let policy = PasswordPolicy::from_str(split.next().unwrap())?;
        let password = split.next().ok_or(ParseProblem::new("password", s.to_string()))?;
        
        Ok(Password { policy, password: password.to_string() })
    }
}

pub fn password_philosophy(input: &str) -> Result<u32, AocError> {
    // Generation
    let mut results: Result<Vec<_>, _> = input.lines().map(|l| Password::from_str(l)).collect();

    // I feel like this is done in a very non-Rustic way but I'm not sure how to fix it
    // and keep error handling. I feel like this would actually be easier with traditional
    // exceptions, but this is probably just that my thinking needs adjusted.
    /*for line in input.lines().map(|l| l.trim()) {
        let split = line.split(": ");
        results.push(Password::from_str(line).unwrap())
}*/
    
    // Solution
    Ok(0)
}
