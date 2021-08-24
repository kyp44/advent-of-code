use nom::{character::complete::digit1, combinator::map};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![],
    "1",
    vec!["1"].answer_vec()
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
        let next = self.current.take();
        self.current = Some("fudge".to_string());
        next
    }
}

pub const SOLUTION: Solution = Solution {
    day: 10,
    name: "Elves Look, Elves Say",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let sequence = Sequence::from_str(input)?;
            for val in sequence.take(5) {
                println!("{}", val);
            }

            Ok(Answer::String(
                Sequence::from_str(input)?.nth(5).unwrap().into(),
            ))
        },
    ],
};
