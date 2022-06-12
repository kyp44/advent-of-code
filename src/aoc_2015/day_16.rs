use std::collections::HashMap;

use maplit::hashmap;
use nom::{
    bytes::complete::{tag, take_until},
    combinator::map,
    multi::separated_list1,
    sequence::{preceded, separated_pair},
};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(40), Unsigned(241)],
    "",
    Vec::new()
    }
}

#[derive(Debug, new)]
struct Sue<'a> {
    #[new(value = "0")]
    number: u16,
    compounds: HashMap<&'a str, u8>,
}
impl<'a> Parseable<'a> for Sue<'a> {
    fn parser(input: &'a str) -> NomParseResult<&str, Self> {
        map(
            separated_pair(
                preceded(tag("Sue "), nom::character::complete::u16),
                tag(": "),
                separated_list1(
                    tag(", "),
                    separated_pair(take_until(":"), tag(": "), nom::character::complete::u8),
                ),
            ),
            |(number, cs)| Sue {
                number,
                compounds: cs.into_iter().collect(),
            },
        )(input.trim())
    }
}

trait Part {
    fn matches(a: &Sue, b: &Sue) -> bool;
}
struct PartA;
impl Part for PartA {
    fn matches(a: &Sue, b: &Sue) -> bool {
        a.compounds
            .keys()
            .filter(|k| b.compounds.contains_key(*k))
            .all(|k| a.compounds.get(k).unwrap() == b.compounds.get(k).unwrap())
    }
}
struct PartB;
impl Part for PartB {
    fn matches(a: &Sue, b: &Sue) -> bool {
        a.compounds
            .keys()
            .filter(|k| b.compounds.contains_key(*k))
            .all(|k| {
                let av = a.compounds.get(k).unwrap();
                let bv = b.compounds.get(k).unwrap();
                if *k == "cats" || *k == "trees" {
                    bv > av
                } else if *k == "pomeranians" || *k == "goldfish" {
                    bv < av
                } else {
                    av == bv
                }
            })
    }
}

struct Problem<'a> {
    sues: Box<[Sue<'a>]>,
}
impl<'a> Problem<'a> {
    fn from_str(s: &'a str) -> AocResult<Self> {
        Ok(Problem {
            sues: Sue::gather(s.lines())?.into_boxed_slice(),
        })
    }

    fn matches<P: Part>(&self) -> impl Iterator<Item = &Sue<'_>> {
        let output = Sue::new(hashmap! {
            "children" => 3,
        "cats" => 7,
        "samoyeds" => 2,
        "pomeranians" => 3,
        "akitas" => 0,
        "vizslas" => 0,
        "goldfish" => 5,
        "trees" => 3,
        "cars" => 2,
        "perfumes" => 1,
        });

        self.sues.iter().filter(move |s| P::matches(&output, s))
    }
}

fn solve<P: Part>(input: &SolverData) -> AocResult<Answer> {
    // Generation
    let problem = Problem::from_str(input.expect_input()?)?;

    // Show matches
    /*for sue in problem.matches::<PartA>() {
        println!("{:?}", sue);
    }*/

    // Process
    let sue = problem.matches::<P>().next().unwrap();
    Ok(Answer::Unsigned(sue.number.into()))
}

pub const SOLUTION: Solution = Solution {
    day: 16,
    name: "Aunt Sue",
    preprocessor: None,
    solvers: &[solve::<PartA>, solve::<PartB>],
};
