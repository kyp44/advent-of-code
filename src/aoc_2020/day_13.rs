use nom::{
    bytes::complete::{is_not, tag},
    character::complete::{digit1, multispace1, space0},
    combinator::map,
    multi::separated_list1,
    sequence::{separated_pair, tuple},
};

use crate::aoc::{ParseResult, Parseable, Solution};

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

pub const SOLUTION: Solution = Solution {
    day: 13,
    name: "Shuttle Search",
    solver: |input| {
        // Generation
        let schedule = Schedule::from_str(input)?;

        // Process
        let time_until = |id: &u64| id - (schedule.earliest_time % *id);
        let bus_id = schedule
            .bus_ids
            .iter()
            .filter_map(|id| *id)
            .min_by(|a, b| time_until(a).cmp(&time_until(b)))
            .unwrap();
        let answers = vec![bus_id * time_until(&bus_id)];

        Ok(answers)
    },
};
