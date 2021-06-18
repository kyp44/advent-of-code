use super::super::aoc::{
    AocError,
    Parseable,
    ParseResult,
    Solution,
};
use nom::{
    character::complete::digit1,
    combinator::map,
    error::context,
};

#[cfg(test)]
mod tests{
    use super::*;
    use crate::solution_test;

    solution_test! {
        "1721
979
366
299
675
1456
",
        vec![514579, 241861950],
        vec![63616, 67877784]
    }
}

type Expense = u32;
impl Parseable for Expense {
    fn parse(input: &str) -> ParseResult<Self> {
        context(
            "expense",
            map(
                digit1,
                |ns: &str| {
                    ns.parse().unwrap()
                }
            )
        )(input.trim())
    }
}

pub const SOLUTION: Solution = Solution {
    day: 1,
    name: "Report Repair",
    solver: |input| {
        // Generation
        let values = Expense::gather(input.lines())?;

        // Processing
        // Part a)
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
        // Part b)
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
    },
};
