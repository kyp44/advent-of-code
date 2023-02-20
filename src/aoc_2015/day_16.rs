use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(40), Unsigned(241)],
    "",
    Vec::new()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use derive_new::new;
    use maplit::hashmap;
    use nom::{
        bytes::complete::{tag, take_until},
        combinator::map,
        multi::separated_list1,
        sequence::{preceded, separated_pair},
    };
    use std::collections::HashMap;

    /// An Aunt Sue and what you remember about her, which can be read from text input.
    #[derive(Debug, new)]
    pub struct Sue<'a> {
        /// The number of this Aunt Sue.
        #[new(value = "0")]
        pub number: u16,
        /// The values of the compounds that you remember for this Aunt Sue.
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

    /// Behavior specific to a particular part of the problem.
    pub trait Part {
        /// Determines whether the known components of an Aunt sue match the MFCSAM output.
        fn matches(output: &Sue, memory: &Sue) -> bool;
    }

    /// Behavior for part one.
    pub struct PartOne;
    impl Part for PartOne {
        fn matches(a: &Sue, b: &Sue) -> bool {
            a.compounds
                .keys()
                .filter(|k| b.compounds.contains_key(*k))
                .all(|k| a.compounds.get(k).unwrap() == b.compounds.get(k).unwrap())
        }
    }

    /// Behavior for part two.
    pub struct PartTwo;
    impl Part for PartTwo {
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

    /// Definition of the problem that can be read from text input.
    pub struct Problem<'a> {
        /// All the things you remember about each Aunt Sue.
        sues: Box<[Sue<'a>]>,
    }
    impl<'a> Problem<'a> {
        /// Parses from text input.
        pub fn from_str(s: &'a str) -> AocResult<Self> {
            Ok(Problem {
                sues: Sue::gather(s.lines())?.into_boxed_slice(),
            })
        }

        /// Returns an [`Iterator`] of Aunt Sues who match the readout from the MFCSAM.
        pub fn matches<P: Part>(&self) -> impl Iterator<Item = &Sue<'_>> {
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

    /// Solves a part of the problem.
    pub fn solve<P: Part>(input: &SolverInput) -> AocResult<Answer> {
        // Generation
        let problem = Problem::from_str(input.expect_input()?)?;

        // Process
        let mut matches = problem.matches::<P>();
        let sue = matches.next().unwrap();
        if matches.next().is_some() {
            Err(AocError::Process("More than one matching Aunt Sue!".into()))
        } else {
            Ok(Answer::Unsigned(sue.number.into()))
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 16,
    name: "Aunt Sue",
    preprocessor: None,
    solvers: &[solve::<PartOne>, solve::<PartTwo>],
};
