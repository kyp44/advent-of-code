use super::aoc::{AocError, ParseResult, Parseable};
use nom::{
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
    use super::super::aoc::test_result;

    #[test]
    fn year_2020_day_01() {
        let input = "1721
979
366
299
675
1456";
        test_result(report_repair(input), vec![514579, 241861950]);
    }
    
    #[test]
    fn year_2020_day_02() {
        let input = "1-3 a: abcde
                     1-3 b: cdefg
                     2-9 c: ccccccccc";
        test_result(password_philosophy(input), vec![2, 1]);
    }

    #[test]
    fn year_2020_day_03() {
        let input = "..##.......
#...#...#..
.#....#..#.
..#.#...#.#
.#...##..#.
..#.##.....
.#.#.#....#
.#........#
#.##...#...
#...##....#
.#..#...#.#";
        test_result(toboggan_trajectory(input), vec![7]);
    }
}

pub fn report_repair(input: &str) -> Result<Vec<u32>, AocError> {
    type Expense = u32;
    impl Parseable for Expense {
        fn parse(input: &str) -> ParseResult<Self> {
            context(
                "expense",
                digit1,
            )(input.trim()).map(|(next, res)| {
                (next, res.parse().unwrap())
            })
        }
    }

    // Generation
    let values = Expense::gather(input.lines())?;

    // Processing
    // Part a
    let mut answers: Vec<u32> = vec![];
    answers.push({
        let mut i = itertools::iproduct!(values.iter(), values.iter());
        loop {
            match i.next() {
                Some((v1, v2)) => {
                    if v1 + v2 == 2020 {
                        break v1*v2;
                    }
                },
                None => {
                    return Err(AocError::Process("No two values add to 2020".to_string()));
                }
            }
        }
    });
    // Part b
    answers.push({
        let mut i = itertools::iproduct!(values.iter(), values.iter(), values.iter());
        loop {
            match i.next() {
                Some((v1, v2, v3)) => {
                    if v1 + v2 + v3 == 2020 {
                        break v1*v2*v3;
                    }
                },
                None => {
                    return Err(AocError::Process("No three values add to 2020".to_string()));
                }
            }
        }
    });


    Ok(answers)
}

pub fn password_philosophy(input: &str) -> Result<Vec<u32>, AocError> {
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

    // Generation
    let passwords = Password::gather(input.lines())?;

    // Processing
    let mut answers = vec![];
    macro_rules! add_filter_count {
        ($a:expr) => {
            answers.push(passwords.iter().filter($a).count() as u32)
        };
    }
    add_filter_count!(|p| p.valid_part_a());
    add_filter_count!(|p| p.valid_part_b());

    Ok(answers)
}

pub fn toboggan_trajectory(input: &str) -> Result<Vec<u32>, AocError> {
    Ok(vec![0])
}
