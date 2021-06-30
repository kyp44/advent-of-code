use std::iter::FilterMap;

use itertools::Itertools;
use nom::{
    bytes::complete::{is_not, tag},
    character::complete::{digit1, multispace1, space0},
    combinator::map,
    multi::separated_list1,
    sequence::{separated_pair, tuple},
};
use num::integer::gcd;

use crate::aoc::{AocError, ParseResult, Parseable, Solution};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![1895],
    "939
    7,13,x,x,59,x,31,19",
        vec![295, 1068781]
    }
}

#[derive(Debug)]
struct Schedule {
    earliest_time: u64,
    bus_ids: Vec<Option<u64>>,
}

impl Parseable for Schedule {
    fn parse(input: &str) -> ParseResult<Self> {
        map(
            separated_pair(
                digit1,
                multispace1,
                separated_list1(tuple((space0, tag(","), space0)), is_not(", \t\n\r")),
            ),
            |(ts, vs): (&str, Vec<&str>)| Schedule {
                earliest_time: ts.parse().unwrap(),
                bus_ids: vs.into_iter().map(|s| s.parse().ok()).collect(),
            },
        )(input)
    }
}

impl Schedule {
    fn valid_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.bus_ids.iter().filter_map(|id| *id)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 13,
    name: "Shuttle Search",
    solver: |input| {
        // Generation
        let schedule = Schedule::from_str(input)?;

        // Process
        let mut answers = vec![];

        // Part a)
        let time_until = |id: &u64| id - (schedule.earliest_time % *id);
        let bus_id = schedule
            .valid_ids()
            .min_by(|a, b| time_until(a).cmp(&time_until(b)))
            .unwrap();
        answers.push(bus_id * time_until(&bus_id));

        // Part b)
        // This problem is effectively the Chinese Remainder Theorem to solve a system
        // of modulo congruences. These can be solved so long as the modulo factors
        // (in our case the set of bus IDs) are all pairwise co-prime. So first we check
        // that this is the case to guarantee that there will be a solution.
        for v in schedule.valid_ids().combinations(2) {
            if gcd(v[0], v[1]) > 1 {
                return Err(AocError::Process(format!(
                    "Part b) may not be solveable because {} and {} are not co-prime",
                    v[0], v[1]
                )));
            }
        }

        Ok(answers)
    },
};
