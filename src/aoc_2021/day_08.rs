use std::collections::HashSet;

use nom::{
    bytes::complete::tag,
    character::complete::{one_of, space1},
    combinator::map,
    multi::{many1, separated_list1},
    sequence::separated_pair,
};

use crate::aoc::{parse::separated, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(123)],
    "acedgfb cdfbe gcdfa fbcad dab cefabd cdfgeb eafb cagedb ab | cdfeb fcadb cdfeb cdbaf",
    vec![123u64].answer_vec()
    }
}

struct Digit {
    segments: HashSet<char>,
}
impl Parseable<'_> for Digit {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(many1(one_of("abcdefg")), |chars| Digit {
            segments: chars.into_iter().collect(),
        })(input)
    }
}

struct Line {
    digits: Box<[Digit]>,
    output: Box<[Digit]>,
}
impl Parseable<'_> for Line {
    fn parser(input: &str) -> NomParseResult<Self> {
        let chars = "abcdefg";

        map(
            separated_pair(
                separated_list1(space1, Digit::parser),
                separated(tag("|")),
                separated_list1(space1, Digit::parser),
            ),
            |(digs, out)| Line {
                digits: digs.into_boxed_slice(),
                output: out.into_boxed_slice(),
            },
        )(input)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 8,
    name: "Seven Segment Search",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let lines = Line::gather(input.lines())?;

            // Process
            Ok(0u64.into())
        },
    ],
};
