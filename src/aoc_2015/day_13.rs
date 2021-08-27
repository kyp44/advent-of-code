use std::collections::HashSet;

use itertools::{process_results, Itertools, ProcessResults};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::map,
    sequence::tuple,
};

use crate::aoc::{prelude::*, trim};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Signed;

    solution_test! {
    vec![Signed(664)],
    "Alice would gain 54 happiness units by sitting next to Bob.
Alice would lose 79 happiness units by sitting next to Carol.
Alice would lose 2 happiness units by sitting next to David.
Bob would gain 83 happiness units by sitting next to Alice.
Bob would lose 7 happiness units by sitting next to Carol.
Bob would lose 63 happiness units by sitting next to David.
Carol would lose 62 happiness units by sitting next to Alice.
Carol would gain 60 happiness units by sitting next to Bob.
Carol would gain 55 happiness units by sitting next to David.
David would gain 46 happiness units by sitting next to Alice.
David would lose 7 happiness units by sitting next to Bob.
David would gain 41 happiness units by sitting next to Carol.",
    vec![330i64].answer_vec()
    }
}

#[derive(Debug)]
struct HappinessChange<'a> {
    person: &'a str,
    next_to: &'a str,
    change: i64,
}
impl<'a> Parseable<'a> for HappinessChange<'a> {
    fn parser(input: &'a str) -> NomParseResult<Self> {
        map(
            tuple((
                take_until(" "),
                trim(tag("would")),
                alt((tag("gain"), tag("lose"))),
                trim(nom::character::complete::i64),
                trim(tag("happiness units by sitting next to")),
                take_until("."),
                tag("."),
            )),
            |(person, _, gl, c, _, next_to, _)| HappinessChange {
                person,
                next_to,
                change: if gl == "lose" { -c } else { c },
            },
        )(input.trim())
    }
}

#[derive(Debug)]
struct Problem<'a> {
    attendees: Box<[&'a str]>,
    changes: Box<[HappinessChange<'a>]>,
}
impl<'a> Problem<'a> {
    fn from_str(s: &'a str) -> AocResult<Self> {
        let changes = HappinessChange::gather(s.lines())?;
        let mut attendees: HashSet<&str> = HashSet::new();

        for change in changes.iter() {
            attendees.insert(change.person);
            attendees.insert(change.next_to);
        }

        Ok(Problem {
            attendees: attendees.into_iter().collect(),
            changes: changes.into_boxed_slice(),
        })
    }
}
impl Problem<'_> {
    fn arrangements(&self) -> impl Iterator<Item = Vec<&str>> {
        let others = &self.attendees[1..];
        others
            .into_iter()
            .copied()
            .permutations(others.len())
            .map(move |mut v| {
                v.insert(0, self.attendees[0]);
                v
            })
    }

    fn arrangement_happiness(&self, arrangement: &[&str]) -> AocResult<i64> {
        let lookup_change = |person: &str, other: &str| -> AocResult<i64> {
            self.changes
                .into_iter()
                .find(|c| c.person == person && c.next_to == other)
                .map(|c| c.change)
                .ok_or_else(|| {
                    AocError::Process(
                        format!(
                            "Could not find happiness change for {} sitting next to {}",
                            person, other
                        )
                        .into(),
                    )
                })
        };

        process_results(
            arrangement
                .into_iter()
                .enumerate()
                .map(|(i, person)| -> Result<_, _> {
                    Ok(lookup_change(
                        person,
                        arrangement[(i + arrangement.len() - 1) % arrangement.len()],
                    )? + lookup_change(person, arrangement[(i + 1) % arrangement.len()])?)
                }),
            |iter: ProcessResults<_, AocError>| iter.sum(),
        )
    }

    fn best_arrangement(&self) -> AocResult<i64> {
        process_results(
            self.arrangements()
                .map(|a| -> Result<_, _> { self.arrangement_happiness(&a) }),
            |iter| iter.max().unwrap(),
        )
    }
}

pub const SOLUTION: Solution = Solution {
    day: 13,
    name: "Knights of the Dinner Table",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let problem = Problem::from_str(input)?;

            // Process
            Ok(problem.best_arrangement()?.into())
        },
    ],
};
