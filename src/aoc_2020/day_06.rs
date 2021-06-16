use super::super::aoc::{
    ParseResult,
    Solution,
};
use nom::{
    Finish,
    branch::alt,
    bytes::complete::is_not,
    character::complete::{line_ending, space0, space1},
    combinator::{all_consuming, map},
    error::context,
    multi::separated_list1,
    sequence::{pair, tuple},
};
use std::collections::HashSet;

#[cfg(test)]
mod tests{
    use super::*;
    use crate::solution_test;

    solution_test! {
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
        vec![11, 6],
        vec![6335, 3392]
    }
}

type Questions = HashSet<char>;

fn make_questions_parser(reducer: fn(Questions, Questions) -> Questions) ->
impl Fn(&str) -> ParseResult<Questions>
{
    move |input| {
        context(
            "questions",
            map(
                separated_list1(
                    alt((pair(space0, line_ending), pair(space1, space0))),
                    is_not(" \t\n\r"),
                ),
                |vec: Vec<&str>| {
                    vec.iter().map(|s| s.chars().collect::<Questions>())
                        .reduce(reducer).unwrap()
                }
            )
        )(input)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 6,
    name: "Custom Customs",
    solver: |input| {
        // Generation
        let parse_input = |reducer| {
            all_consuming(
                separated_list1(
                    tuple((space0, line_ending, space0, line_ending)),
                    make_questions_parser(reducer),
                )
            )(input.trim_end()).finish().map(|(_, pd)| pd)
        };
        let part_questions = vec![
            parse_input(|a, b| {
                a.union(&b).copied().collect()
            })?,
            parse_input(|a, b| {
                a.intersection(&b).copied().collect()
            })?,
        ];

        // Processing
        let answers = part_questions.iter().map(|v| v.iter().map(|q| q.len() as u32).sum()).collect();
        
        Ok(answers)
    }
};
