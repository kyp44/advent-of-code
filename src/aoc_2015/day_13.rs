use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Signed;

    solution_tests! {
        example {
            input = "Alice would gain 54 happiness units by sitting next to Bob.
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
David would gain 41 happiness units by sitting next to Carol.";
            answers = vec![330i64, 286].answer_vec();
        }
        actual_answers = vec![Signed(664), Signed(640)];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use bare_metal_modulo::{MNum, ModNum};
    use itertools::{process_results, Itertools, ProcessResults};
    use nom::{
        branch::alt,
        bytes::complete::{tag, take_until},
        combinator::map,
        sequence::tuple,
    };
    use std::collections::HashSet;

    /// A change in happiness for a holiday feast attendee that can be parsed from text input.
    #[derive(Debug)]
    struct SeatingPreference<'a> {
        /// Name of the person whose happiness is changed.
        person: &'a str,
        /// The name of the person next them that is causing the change.
        next_to: &'a str,
        /// Amount of happiness change.
        change: i64,
    }
    impl<'a> Parseable<'a> for SeatingPreference<'a> {
        fn parser(input: &'a str) -> NomParseResult<&str, Self> {
            map(
                tuple((
                    take_until(" "),
                    trim(false, tag("would")),
                    alt((tag("gain"), tag("lose"))),
                    trim(false, nom::character::complete::i64),
                    trim(false, tag("happiness units by sitting next to")),
                    take_until("."),
                    tag("."),
                )),
                |(person, _, gl, c, _, next_to, _)| SeatingPreference {
                    person,
                    next_to,
                    change: if gl == "lose" { -c } else { c },
                },
            )(input.trim())
        }
    }

    /// Problem definition that can be parsed from text input.
    #[derive(Debug)]
    pub struct Problem<'a> {
        /// List of all attendee names.
        attendees: Vec<&'a str>,
        /// Change in happiness values caused by seating people next to each other.
        preferences: Vec<SeatingPreference<'a>>,
    }
    impl<'a> Problem<'a> {
        /// Parses the problem from text input.
        pub fn from_str(s: &'a str) -> AocResult<Self> {
            let preferences = SeatingPreference::gather(s.lines())?;
            let mut attendees: HashSet<&str> = HashSet::new();

            for change in preferences.iter() {
                attendees.insert(change.person);
                attendees.insert(change.next_to);
            }

            Ok(Problem {
                attendees: attendees.into_iter().collect(),
                preferences,
            })
        }

        /// Returns an [`Iterator`] over all possible seating arrangements.
        fn arrangements(&self) -> impl Iterator<Item = Vec<&str>> {
            let others = &self.attendees[1..];
            others
                .iter()
                .copied()
                .permutations(others.len())
                .map(move |mut v| {
                    v.insert(0, self.attendees[0]);
                    v
                })
        }

        /// Calculates the total change in happiness for a particular arrangement.
        fn arrangement_happiness(&self, arrangement: &[&str]) -> AocResult<i64> {
            let lookup_change = |person: &str, other: &str| -> AocResult<i64> {
                self.preferences
                    .iter()
                    .find(|c| c.person == person && c.next_to == other)
                    .map(|c| c.change)
                    .ok_or_else(|| {
                        AocError::Process(
                            format!(
                            "Could not find happiness change for {person} sitting next to {other}"
                        )
                            .into(),
                        )
                    })
            };

            process_results(
                arrangement
                    .iter()
                    .enumerate()
                    .map(|(i, person)| -> Result<_, _> {
                        let idx = ModNum::new(i, arrangement.len());
                        Ok(lookup_change(person, arrangement[(idx - 1).a()])?
                            + lookup_change(person, arrangement[(idx + 1).a()])?)
                    }),
                |iter: ProcessResults<_, AocError>| iter.sum(),
            )
        }

        /// Determines the arrangement that maximizes the total change in happiness.
        pub fn best_arrangement(&self) -> AocResult<i64> {
            process_results(
                self.arrangements()
                    .map(|a| -> Result<_, _> { self.arrangement_happiness(&a) }),
                |iter| iter.max().unwrap_or(0),
            )
        }

        /// Adds an attendee who has completely neutral seating preferences as do others towards them.
        pub fn add_attendee(&mut self, name: &'a str) {
            // Add neutral seating preferences
            for att in self.attendees.iter() {
                self.preferences.push(SeatingPreference {
                    person: name,
                    next_to: att,
                    change: 0,
                });
                self.preferences.push(SeatingPreference {
                    person: att,
                    next_to: name,
                    change: 0,
                });
            }

            // Add attendee
            self.attendees.push(name);
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 13,
    name: "Knights of the Dinner Table",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let problem = Problem::from_str(input.expect_input()?)?;

            // Process
            Ok(problem.best_arrangement()?.into())
        },
        // Part two
        |input| {
            // Generation
            let mut problem = Problem::from_str(input.expect_input()?)?;

            // Process
            problem.add_attendee("Self");
            //println!("Solution: {}", problem.best_arrangement()?);
            Ok(problem.best_arrangement()?.into())
        },
    ],
};
