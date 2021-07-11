use std::{collections::HashSet, ops::RangeInclusive};

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
    vec![Some(71), Some(222)],
    "class: 0-1 or 4-19
row: 0-5 or 8-19
seat: 0-13 or 16-19

your ticket:
11,12,13

nearby tickets:
3,9,18
15,1,5
5,14,9",
    vec![None, Some(222)]
    }
}

#[derive(Debug)]
struct Field {
    name: String,
    valid_ranges: Vec<RangeInclusive<u32>>,
}
impl Parseable for Field {
    fn parser(input: &str) -> ParseResult<Self> {
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

impl Parseable for Ticket {
    fn parser(input: &str) -> ParseResult<Self> {
        Ok((
            "",
            Ticket {
                field_values: u32::from_csv(input).map_err(|e| nom::Err::Failure(e))?,
            },
        ))
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

        // Parse fields
        let fields = Field::gather(sections[0].lines())?;
        let num_fields = fields.len();

        let verify_fields = |name: &str, ticket: &Ticket| match ticket.field_values.len() {
            n if n == num_fields => Ok(()),
            _ => Err(AocError::InvalidInput(format!(
                "{} ticket has {} fields when {} are expected",
                name,
                ticket.field_values.len(),
                num_fields
            ))),
        };

        // Parse your ticket and verify the number of fields
        let your_ticket =
            preceded(tuple((tag("your ticket:"), multispace1)), Ticket::parser)(sections[1])
                .discard_input()?;
        verify_fields("Your", &your_ticket)?;

        // Parse nearby tickets and verify the number of fields
        let nearby_tickets =
            preceded(tuple((tag("nearby tickets:"), multispace1)), rest)(sections[2])
                .discard_input()?
                .lines()
                .map(|l| Ticket::from_str(l))
                .collect::<Result<Vec<Ticket>, ParseError>>()?;
        for ticket in nearby_tickets.iter() {
            verify_fields("A nearby", ticket)?;
        }

        Ok(Problem {
            fields,
            your_ticket,
            nearby_tickets,
        })
    }
}
impl Problem {
    fn is_valid(&self, value: &u32) -> bool {
        self.fields.iter().any(|f| f.is_valid(value))
    }

    // NOTE: Made a post about how to accomplish returning an Iterator here instead
    // of a collected Vec.
    // See: https://users.rust-lang.org/t/returning-iterator-seemingly-requiring-multiple-liftetimes/62179/3
    // This was later simplified to use u32 instead of &u32 in the Iterator.
    fn invalid_fields<'a>(&'a self, ticket: &'a Ticket) -> impl Iterator<Item = u32> + 'a {
        ticket
            .field_values
            .iter()
            .filter_map(move |v| if self.is_valid(v) { None } else { Some(*v) })
    }

    fn all_invalid_fields<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        self.nearby_tickets
            .iter()
            .flat_map(move |t| self.invalid_fields(t))
    }

    fn solve(&self) {
        // First get a set of all invalid values
        let invalid_values: HashSet<u32> = self.all_invalid_fields().collect();

        println!("TODO {:?}", invalid_values);
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

            // Process
            Ok(problem.all_invalid_fields().sum::<u32>().into())
        },
        // Part b)
        |input| {
            // Generation
            let problem = Problem::from_str(input)?;

            problem.solve();
            Ok(0)
        },
    ],
};
