use std::convert::TryInto;

use itertools::{process_results, ProcessResults};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, none_of},
    combinator::value,
    multi::many0,
    sequence::{delimited, preceded, tuple},
    Finish,
};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(1333), Unsigned(2046)],
    "\"\"
\"abc\"
\"aaa\\\"aaa\"
\"\\x27\"",
    vec![12u64, 19].answer_vec()
    }
}

#[derive(new)]
struct ListString<'a> {
    literal: &'a str,
}
impl<'a> ListString<'a> {
    fn escaped(&self) -> AocResult<String> {
        delimited(
            tag("\""),
            many0(alt((
                preceded(
                    tag("\\"),
                    alt((
                        value('\\', tag("\\")),
                        value('"', tag("\"")),
                        value('-', tuple((tag("x"), anychar, anychar))),
                    )),
                ),
                none_of("\""),
            ))),
            tag("\""),
        )(self.literal)
        .finish()
        .discard_input()
        .map(|s| s.into_iter().collect())
        .map_err(|e: NomParseError| e.into())
    }

    fn encoded(&self) -> String {
        format!(
            "\"{}\"",
            self.literal.replace("\\", "\\\\").replace("\"", "\\\"")
        )
    }
}

struct List<'a> {
    list_strings: Vec<ListString<'a>>,
}
impl<'a> List<'a> {
    fn from_str(s: &'a str) -> Self {
        List {
            list_strings: s.lines().map(|s| ListString::new(s.trim())).collect(),
        }
    }
}

fn solution(list: &List, f: impl Fn(&ListString) -> AocResult<usize>) -> AocResult<Answer> {
    /*for ls in list.list_strings.iter() {
        println!("<{}>' <{}> <{}>", ls.literal, ls.escaped()?, ls.encoded());
    }*/

    Ok(Answer::Unsigned(
        process_results(
            list.list_strings.iter().map(f),
            |iter: ProcessResults<_, AocError>| iter.sum::<usize>(),
        )?
        .try_into()
        .unwrap(),
    ))
}

pub const SOLUTION: Solution = Solution {
    day: 8,
    name: "Matchsticks",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let list = List::from_str(input);

            // Process
            solution(&list, |ls| Ok(ls.literal.len() - ls.escaped()?.len()))
        },
        // Part b)
        |input| {
            // Generation
            let list = List::from_str(input);

            // Process
            solution(&list, |ls| Ok(ls.encoded().len() - ls.literal.len()))
        },
    ],
};
