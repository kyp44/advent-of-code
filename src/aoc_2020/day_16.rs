use std::{ops::RangeInclusive, str::FromStr};

use nom::{
    bytes::complete::{is_not, tag},
    character::complete::{digit1, multispace1},
    combinator::{map, rest},
    multi::separated_list1,
    sequence::{preceded, separated_pair, tuple},
};

use crate::aoc::{AocError, DiscardInput, ParseError, ParseResult, Parseable, Solution};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![29851],
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
impl Field {
    fn is_valid(&self, value: &u32) -> bool {
        self.valid_ranges.iter().any(|r| r.contains(value))
    }
}

#[derive(Debug)]
struct Ticket {
    field_values: Vec<u32>,
}
impl FromStr for Ticket {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Ticket {
            field_values: u32::from_csv(s)?,
        })
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
            your_ticket: preceded(tuple((tag("your ticket:"), multispace1)), rest)(sections[1])
                .discard_input()?
                .parse()?,
            nearby_tickets: preceded(tuple((tag("nearby tickets:"), multispace1)), rest)(
                sections[2],
            )
            .discard_input()?
            .lines()
            .map(|l| l.parse())
            .collect::<Result<Vec<Ticket>, ParseError>>()?,
        })
    }
}
impl Problem {
    fn is_valid(&self, value: &u32) -> bool {
        self.fields.iter().any(|f| f.is_valid(value))
    }

    fn invalid_fields(&self, ticket: &Ticket) -> Vec<u32> {
        ticket
            .field_values
            .iter()
            .filter_map(|v| if self.is_valid(v) { None } else { Some(*v) })
            .collect()
    }

    /*fn invalid_fieldss<'a, 'b>(&'a self, ticket: &'b Ticket) -> impl Iterator<Item = &'b u32> {
        ticket
            .field_values
            .iter()
            .filter_map(|v| if self.is_valid(v) { None } else { Some(v) })
    }*/
}

pub const SOLUTION: Solution = Solution {
    day: 16,
    name: "Ticket Translation",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let problem = Problem::from_str(input)?;

            // Process
            Ok(problem
                .nearby_tickets
                .iter()
                .flat_map(|t| problem.invalid_fields(t))
                .sum::<u32>()
                .into())
        },
    ],
};
