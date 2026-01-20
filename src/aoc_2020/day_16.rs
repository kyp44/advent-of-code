use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use Answer::Unsigned;
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "class: 1-3 or 5-7
row: 6-11 or 33-44
seat: 13-40 or 45-50

your ticket:
7,1,14

nearby tickets:
7,3,47
40,4,50
55,2,20
38,6,12";
            // Solution: row, class, seat
            answers = &[Some(Unsigned(71)), None];
        }
        example {
            input = "class: 0-1 or 4-19
row: 0-5 or 8-19
seat: 0-13 or 16-19

your ticket:
11,12,13

nearby tickets:
3,9,18
15,1,5
5,14,9";
            // Solution: row, class, seat
            answers = &[None, Some(Unsigned(1))];
        }
        actual_answers = unsigned![29851, 3029180675981];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use nom::{
        Finish,
        bytes::complete::{is_not, tag},
        character::complete::multispace1,
        combinator::{map, rest},
        multi::separated_list1,
        sequence::{pair, preceded, separated_pair},
    };
    use std::{collections::HashSet, hash::Hash, ops::RangeInclusive};

    /// A ticket field, which can be parsed from text input.
    #[derive(Debug, Eq)]
    struct Field {
        /// The name of the field.
        name: String,
        /// Valid inclusive ranges of the field value.
        valid_ranges: Vec<RangeInclusive<u32>>,
    }
    impl Parsable<'_> for Field {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            use nom::character::complete::u32 as cu32;
            map(
                separated_pair(
                    is_not(":"),
                    tag(": "),
                    separated_list1(tag(" or "), separated_pair(cu32, tag("-"), cu32)),
                ),
                |(name, v): (&str, Vec<(u32, u32)>)| Field {
                    name: name.to_string(),
                    valid_ranges: v.into_iter().map(|(sa, sb)| sa..=sb).collect(),
                },
            )
            .parse(input.trim())
        }
    }
    impl Field {
        /// Tests whether a value is valid for this field.
        fn is_valid(&self, value: &u32) -> bool {
            self.valid_ranges.iter().any(|r| r.contains(value))
        }
    }
    impl PartialEq for Field {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
        }
    }
    impl Hash for Field {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.name.hash(state);
        }
    }

    /// A ticket with ordered but unknown fields, which can be parsed from text input.
    #[derive(Debug)]
    struct Ticket {
        /// List of field values in order.
        field_values: Vec<u32>,
    }
    impl Parsable<'_> for Ticket {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            Ok((
                "",
                Ticket {
                    field_values: u32::from_csv(input).map_err(nom::Err::Failure)?,
                },
            ))
        }
    }

    /// The problem definition, which can be parsed from text input.
    #[derive(Debug)]
    pub struct Problem {
        /// List of fields with their valid values.
        fields: Vec<Field>,
        /// Your ticket.
        your_ticket: Ticket,
        /// List of nearby tickets.
        nearby_tickets: Vec<Ticket>,
    }
    impl FromStr for Problem {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let sections = s.sections(3)?;

            // Parse fields
            let fields = Field::gather(sections[0].lines())?;
            let num_fields = fields.len();

            let verify_fields = |name: &str, ticket: &Ticket| match ticket.field_values.len() {
                n if n == num_fields => Ok(()),
                _ => Err(AocError::InvalidInput(
                    format!(
                        "{} ticket has {} fields when {} are expected",
                        name,
                        ticket.field_values.len(),
                        num_fields
                    )
                    .into(),
                )),
            };

            // Parse your ticket and verify the number of fields
            let your_ticket = preceded((tag("your ticket:"), multispace1), Ticket::parser)
                .parse(sections[1])
                .finish()
                .discard_input()?;
            verify_fields("Your", &your_ticket)?;

            // Parse nearby tickets and verify the number of fields
            let result: NomParseResult<&str, _> =
                preceded(pair(tag("nearby tickets:"), multispace1), rest).parse(sections[2]);
            let nearby_tickets = result
                .finish()
                .discard_input()?
                .lines()
                .map(Ticket::from_str)
                .collect::<Result<Vec<Ticket>, NomParseError>>()?;
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
        /// Checks whether a value is valid for at least one field.
        fn is_valid(&self, value: &u32) -> bool {
            self.fields.iter().any(|f| f.is_valid(value))
        }

        /// Returns an [`Iterator`] over field values from a ticket that are not
        /// valid for any field.
        ///
        /// NOTE: Made [a post](https://users.rust-lang.org/t/returning-iterator-seemingly-requiring-multiple-liftetimes/62179/3)
        /// about how to accomplish returning an Iterator here instead of a collected [`Vec`].
        /// This was later simplified to use [`u32`] instead of `&u32` in the Iterator.
        fn invalid_fields<'a>(&'a self, ticket: &'a Ticket) -> impl Iterator<Item = u32> + 'a {
            ticket
                .field_values
                .iter()
                .filter_map(move |v| if self.is_valid(v) { None } else { Some(*v) })
        }

        /// Returns an [`Iterator`] over field values from all tickets that are not
        /// valid for any field.
        pub fn all_invalid_fields(&self) -> impl Iterator<Item = u32> + '_ {
            self.nearby_tickets
                .iter()
                .flat_map(move |t| self.invalid_fields(t))
        }

        /// Determines the order of the fields on the tickets, returning a list
        /// of the fields in the correct order that matches each ticket.
        fn solve(&self) -> AocResult<Vec<&Field>> {
            // First get a set of all invalid values
            let invalid_values: HashSet<u32> = self.all_invalid_fields().collect();

            // Next determine possible Fields for each field position,
            // i.e. those Fields for which every non-completely-invalid field is valid.
            let mut possible_fields: Vec<HashSet<_>> = (0..self.fields.len())
                .map(|i| {
                    self.fields
                        .iter()
                        .filter(|field| {
                            self.nearby_tickets
                                .iter()
                                .filter_map(|t| {
                                    let val = t.field_values[i];
                                    if invalid_values.contains(&val) {
                                        None
                                    } else {
                                        Some(val)
                                    }
                                })
                                .all(|val| field.is_valid(&val))
                        })
                        .collect()
                })
                .collect();

            // Now eliminate until each position has only one possible field
            Ok(loop {
                let single_fields: Vec<&Field> = possible_fields
                    .iter()
                    .filter_map(|fields| {
                        if fields.len() == 1 {
                            Some(*fields.iter().next().unwrap())
                        } else {
                            None
                        }
                    })
                    .collect();
                let multi_fields: Vec<&mut HashSet<&Field>> = possible_fields
                    .iter_mut()
                    .filter(|fields| fields.len() > 1)
                    .collect();
                let len = multi_fields.len();
                if len == 0 {
                    // Our work is done, remove the HashSets to get the single element
                    break possible_fields
                        .into_iter()
                        .map(|mut fields| fields.drain().next().unwrap())
                        .collect();
                } else if len == self.fields.len() {
                    // No deduction is possible, at least not with this simple algorithm
                    return Err(AocError::Process(
                        "No position has only one possible field so a solution may not be possible"
                            .into(),
                    ));
                }

                // For each set remove all the fields whos positions are known
                for fields in multi_fields {
                    for field in single_fields.iter() {
                        fields.remove(field);
                    }
                }
            })
        }

        /// Solves for field order and returns the product of all values on your
        /// ticket for fields that start with the specified string.
        pub fn your_field_value_product(&self, starts_with: &str) -> AocResult<u64> {
            let fields = self.solve()?;
            //println!("Solution: {:?}", fields);

            // Now get the desired fields
            Ok(fields
                .into_iter()
                .zip(self.your_ticket.field_values.iter())
                .filter_map(|(f, v)| {
                    if f.name.starts_with(starts_with) {
                        Some(*v)
                    } else {
                        None
                    }
                })
                .map(|v| -> u64 { v.into() })
                .product())
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 16,
    name: "Ticket Translation",
    preprocessor: Some(|input| Ok(Box::new(Problem::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<Problem>()?
                    .all_invalid_fields()
                    .sum::<u32>()
                    .into(),
            ))
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Problem>()?
                .your_field_value_product("departure")?
                .into())
        },
    ],
};
