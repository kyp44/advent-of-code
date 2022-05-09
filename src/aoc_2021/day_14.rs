use std::str::FromStr;

use nom::{character::complete::alphanumeric1, combinator::map, sequence::{separated_pair, pair}, bytes::complete::tag};

use crate::aoc::{prelude::*, parse::{single_alphanumeric, trim}};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;
    
    solution_test! {
        vec![Unsigned(123)],
    "NNCB

CH -> B
HH -> N
CB -> H
NH -> C
HB -> C
HC -> B
HN -> C
NN -> C
BH -> H
NC -> B
NB -> B
BN -> B
BB -> N
BC -> B
CC -> N
CN -> C",
        vec![123u64].answer_vec()
    }
}

#[derive(Debug)]
struct Formula {
    elements: Vec<char>,
}
impl Parseable<'_> for Formula {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(
            alphanumeric1,
            |s: &str| {
                Self {
                    elements: s.chars().collect(),
                }
            }
        )(input)
    }
}

#[derive(Debug)]
struct PairInsertion {
    left: char,
    right: char,
    insert: char,
}
impl Parseable<'_> for PairInsertion {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(
            separated_pair(
                pair(single_alphanumeric, single_alphanumeric),
                trim(tag("->")),
                single_alphanumeric, 
            ),
            |(lr, insert)| {
                Self {
                    left: lr.0,
                    right: lr.1,
                    insert,
                }
            }
        )(input)
    }
}

#[derive(Debug)]
struct PolymerBuilder {
    template: Formula,
    pair_insertions: Box<[PairInsertion]>,
}
impl FromStr for PolymerBuilder {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
            let secs = s.sections(2)?;

            Ok(Self {
                template: Formula::from_str(secs[0])?,
                pair_insertions: PairInsertion::gather(secs[1].lines())?.into_boxed_slice(),
            })
    }
}

pub const SOLUTION: Solution = Solution {
    day: 14,
    name: "Extended Polymerization",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let builder = PolymerBuilder::from_str(input)?;

            println!("TODO: {:?}", builder);
            
            // Process
            Ok(0u64.into())
        },
    ],
};
