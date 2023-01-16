use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
        vec![Unsigned(970), Unsigned(587)],
        "BFFFBBFRRR
FFFBBBFRRR
BBFFBBFRLL",
        vec![Some(Unsigned(820)), None]
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use nom::{
        bytes::complete::take_while_m_n,
        combinator::{all_consuming, map},
        error::context,
        sequence::pair,
    };

    /// The coordinates of a set on the plane, which can be parsed from text input.
    #[derive(Debug)]
    pub struct Seat {
        /// Seat row.
        row: u32,
        /// Seat column.
        column: u32,
    }
    impl Parseable<'_> for Seat {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            /// Sub-function of [Seat::parser].
            /// Creates a parser closure for a letter-coded binary value of a
            /// certain number of bits.
            fn binary_parser(
                bit0: char,
                bit1: char,
                len: usize,
            ) -> impl FnMut(&str) -> NomParseResult<&str, u32> {
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
        /// Returns the ID number of this seat.
        pub fn id(&self) -> u32 {
            self.row * 8 + self.column
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 5,
    name: "Binary Boarding",
    preprocessor: Some(|input| {
        let seats = Seat::gather(input.lines())?;

        Ok(Box::new({
            let mut ids = seats.iter().map(Seat::id).collect::<Vec<u32>>();
            ids.sort_unstable();
            ids
        })
        .into())
    }),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<Vec<u32>>()?
                    .iter()
                    .fold(0, |o, n| o.max(*n))
                    .into(),
            ))
        },
        // Part two
        |input| {
            // Process
            let ids = input.expect_data::<Vec<u32>>()?;
            let missing_id = match ids
                .iter()
                .find(|id| !ids.contains(&(*id + 1)) && ids.contains(&(*id + 2)))
            {
                Some(id) => *id + 1,
                None => 0,
            };
            Ok(Answer::Unsigned(missing_id.into()))
        },
    ],
};
