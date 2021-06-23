use super::super::aoc::{ParseResult, Solution};
use nom::{
    branch::alt,
    bytes::complete::is_not,
    character::complete::{line_ending, space0, space1},
    combinator::{all_consuming, map},
    error::context,
    multi::separated_list1,
    sequence::{pair, tuple},
    Finish,
};
use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
        vec![6335, 3392],
        "abc

a
b
c

ab
ac

a
a
a
a

b
",
        vec![11, 6]
    }
}

type Questions = HashSet<char>;

// Note this could have been done per the solution to my StackExchange question by adding the Copy trait bound:
// https://stackoverflow.com/questions/68007717/rust-nested-closure-moves-and-multiple-owners
// However, this results in a different error about the the closure type when calling this with two different closures.
// It sounds like this could be fixed by "boxing your closure and/or using it as a trait object", but it's
// probably just more efficient to accept an fn instead (certainly rather than boxing).
fn make_questions_parser(
    reducer: fn(Questions, Questions) -> Questions,
) -> impl Fn(&str) -> ParseResult<Questions> {
    move |input| {
        context(
            "questions",
            map(
                separated_list1(
                    alt((pair(space0, line_ending), pair(space1, space0))),
                    is_not(" \t\n\r"),
                ),
                |vec: Vec<&str>| {
                    vec.iter()
                        .map(|s| s.chars().collect::<Questions>())
                        .reduce(reducer)
                        .unwrap()
                },
            ),
        )(input)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 6,
    name: "Custom Customs",
    solver: |input| {
        // Generation
        let parse_input = |reducer| {
            all_consuming(separated_list1(
                tuple((space0, line_ending, space0, line_ending)),
                make_questions_parser(reducer),
            ))(input.trim_end())
            .finish()
            .map(|(_, pd)| pd)
        };
        let part_questions = vec![
            parse_input(|a: Questions, b: Questions| a.union(&b).copied().collect())?,
            parse_input(|a: Questions, b: Questions| a.intersection(&b).copied().collect())?,
        ];

        // Processing
        let answers = part_questions
            .iter()
            .map(|v| v.iter().map(|q| q.len() as u64).sum())
            .collect();

        Ok(answers)
    },
};
