use std::ops::RangeInclusive;

use nom::{
    bytes::complete::{is_not, tag},
    character::complete::digit1,
    combinator::map,
    multi::separated_list1,
    sequence::separated_pair,
};

use crate::aoc::{AocError, ParseResult, Parseable, Solution};
#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![],
"class: 1-3 or 5-7
row: 6-11 or 33-44
seat: 13-40 or 45-50

your ticket:
7,1,14

nearby tickets:
7,3,47
40,4,50
55,2,20
38,6,12",
    vec![Some(71)]
    }
}

#[derive(Debug)]
struct Field {
    name: String,
    valid_ranges: Vec<RangeInclusive<u32>>,
}
impl Parseable for Field {
    fn parse(input: &str) -> ParseResult<Self> {
        map(
            separated_pair(
                is_not(":"),
                tag(": "),
                separated_list1(tag(" or "), separated_pair(digit1, tag("-"), digit1)),
            ),
            |(name, v): (&str, Vec<(&str, &str)>)| Field {
                name: name.to_string(),
                valid_ranges: v
                    .iter()
                    .map(|(sa, sb)| sa.parse().unwrap()..=sb.parse().unwrap())
                    .collect(),
            },
        )(input.trim())
    }
}

#[derive(Debug)]
struct Ticket {
    field_values: Vec<u32>,
}
impl Parseable for Ticket {
    fn parse(input: &str) -> ParseResult<Self> {
        todo!()
    }
}

#[derive(Debug)]
struct Problem {
    fields: Vec<Field>,
    your_ticket: Ticket,
    nearby_tickets: Vec<Ticket>,
}
impl Problem {
    fn from_str(s: &str) -> Result<Problem, AocError> {
        let sections: Vec<&str> = s.split("\n\n").collect();
        if sections.len() != 3 {
            return Err(AocError::InvalidInput(
                "Input does not contain exactly the three expected sections".to_string(),
            ));
        }
        Ok(Problem {
            fields: Field::gather(sections[0].lines())?,
            your_ticket: Ticket {
                field_values: vec![],
            },
            nearby_tickets: vec![],
        })
    }
}

pub const SOLUTION: Solution = Solution {
    day: 16,
    name: "Ticket Translation",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let problem = Problem::from_str(input)?;
            println!("TODO: {:?}", problem);

            // Process
            Ok(0)
        },
    ],
};
