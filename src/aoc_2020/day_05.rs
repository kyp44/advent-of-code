use crate::aoc::AocError;

use super::super::aoc::{ParseResult, Parseable, Solution};
use nom::{
    bytes::complete::take_while_m_n,
    combinator::{all_consuming, map},
    error::context,
    sequence::pair,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
        vec![970, 587],
        "BFFFBBFRRR
FFFBBBFRRR
BBFFBBFRLL",
        vec![Some(820), None]
    }
}

#[derive(Debug)]
struct Seat {
    row: u32,
    column: u32,
}

impl Parseable for Seat {
    fn parser(input: &str) -> ParseResult<Self> {
        // Creates a parser closure for a letter-coded binary value of a
        // certain number of bits.
        fn binary_parser(
            bit0: char,
            bit1: char,
            len: usize,
        ) -> impl FnMut(&str) -> ParseResult<u32> {
            move |input| {
                map(
                    take_while_m_n(len, len, |c: char| c == bit0 || c == bit1),
                    |s: &str| {
                        let bs: String = s
                            .chars()
                            .map(|c| if c == bit0 { '0' } else { '1' })
                            .collect();
                        u32::from_str_radix(&bs, 2).unwrap()
                    },
                )(input)
            }
        }

        context(
            "seat",
            map(
                all_consuming(pair(binary_parser('F', 'B', 7), binary_parser('L', 'R', 3))),
                |(row, column)| Seat { row, column },
            ),
        )(input.trim())
    }
}

impl Seat {
    fn id(&self) -> u32 {
        self.row * 8 + self.column
    }
}

fn get_ids(input: &str) -> Result<Vec<u32>, AocError> {
    let seats = Seat::gather(input.lines())?;

    // Process
    Ok({
        let mut ids = seats.iter().map(Seat::id).collect::<Vec<u32>>();
        ids.sort_unstable();
        ids
    })
}

pub const SOLUTION: Solution = Solution {
    day: 5,
    name: "Binary Boarding",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let ids = get_ids(input)?;

            Ok(ids.iter().fold(0, |o, n| o.max(*n)).into())
        },
        // Part b)
        |input| {
            // Generation
            let ids = get_ids(input)?;

            let missing_id = match ids
                .iter()
                .find(|id| !ids.contains(&(*id + 1)) && ids.contains(&(*id + 2)))
            {
                Some(id) => *id + 1,
                None => 0,
            };
            Ok(missing_id.into())
        },
    ],
};
