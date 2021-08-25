use std::convert::TryInto;

use nom::{character::complete::digit1, combinator::map};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expensive_test, solution_test};
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

struct Sequence {
    current: Option<String>,
}
impl Parseable<'_> for Sequence {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(digit1, |ds: &str| Sequence {
            current: Some(ds.to_string()),
        })(input.trim())
    }
}
impl Iterator for Sequence {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.current.take().unwrap();

        self.current = Some(
            next.split_runs()
                .map(|s| format!("{}{}", s.len(), s.chars().next().unwrap()))
                .collect(),
        );
        Some(next)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 10,
    name: "Elves Look, Elves Say",
    solvers: &[
        // Part a)
        |input| {
            //println!("{}", Sequence::from_str(input)?.nth(40).unwrap().len());
            Ok(Answer::Unsigned(
                Sequence::from_str(input)?
                    .nth(40)
                    .unwrap()
                    .len()
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part b)
        |input| {
            //println!("{}", Sequence::from_str(input)?.nth(50).unwrap().len());
            Ok(Answer::Unsigned(
                Sequence::from_str(input)?
                    .nth(50)
                    .unwrap()
                    .len()
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
