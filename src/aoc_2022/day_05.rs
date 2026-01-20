use std::str::FromStr;

use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "    [D]
[N] [C]    
[Z] [M] [P]
 1   2   3 

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2";
            answers = string!["CMZ", "MCD"];
        }
        actual_answers = string!["VGBBJCRMN", "LBBVJBRMH"];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::{separated, trim};
    use indexmap::IndexMap;
    use nom::{
        Finish,
        branch::alt,
        bytes::complete::tag,
        combinator::map,
        multi::separated_list1,
        sequence::{delimited, preceded},
    };
    use std::str::FromStr;

    /// A cell when parsing the stack data as a grid.
    #[derive(Debug)]
    enum StackCell {
        /// A space with no crate.
        Space,
        /// A crate with its letter.
        Crate(char),
        /// A numerical stack label.
        Label(u8),
    }
    impl Parsable<'_> for StackCell {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            alt((
                map(tag("   "), |_| StackCell::Space),
                map(
                    delimited(tag("["), nom::character::complete::anychar, tag("]")),
                    StackCell::Crate,
                ),
                map(
                    delimited(tag(" "), nom::character::complete::u8, tag(" ")),
                    StackCell::Label,
                ),
            ))
            .parse(input)
        }
    }

    /// A collection of stacks of crates.
    #[derive(Debug, Clone)]
    pub struct Stacks {
        /// The map of stacks with the key being the numerical stack label,
        /// and the value being the stack of crate letters.
        map: IndexMap<u8, Vec<char>>,
    }
    impl FromStr for Stacks {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // Parse raw rows
            let mut rows = s
                .lines()
                .map(|line| {
                    separated_list1(tag(" "), StackCell::parser)
                        .parse(line)
                        .finish()
                        .discard_input()
                })
                .collect::<Result<Vec<_>, _>>()?;

            // Pull out the label row
            let label_row = rows
                .pop()
                .ok_or_else(|| AocError::InvalidInput("The stack section is empty!".into()))?;

            // Build stacks
            let mut map = IndexMap::new();
            for (col, label_cell) in label_row.into_iter().enumerate() {
                if let StackCell::Label(label) = label_cell {
                    let mut stack: Vec<_> = rows
                        .iter()
                        .filter_map(|row| {
                            if col < row.len() {
                                if let StackCell::Crate(c) = row[col] {
                                    Some(c)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();

                    stack.reverse();
                    map.insert(label, stack);
                } else {
                    return Err(AocError::InvalidInput(
                        "The last row contains things things other than just labels".into(),
                    ));
                }
            }

            Ok(Self { map })
        }
    }
    impl Stacks {
        /// Returns a mutable stack, or the appropriate error if the stack is not found.
        pub fn get_stack(&mut self, num: u8) -> AocResult<&mut Vec<char>> {
            self.map
                .get_mut(&num)
                .ok_or_else(|| AocError::Process(format!("Stack '{}' not found", num).into()))
        }

        /// Creates a string from the letters of the crates on the top of each stack,
        /// in order.
        pub fn top_crates(&self) -> String {
            self.map.values().filter_map(|stack| stack.last()).collect()
        }
    }

    /// A move from one crate stack to another.
    #[derive(Debug, Clone)]
    pub struct Move {
        /// The number of crates to move.
        number: usize,
        /// The stack label from which to move the crates.
        from: u8,
        /// The stack label to which to move the crates.
        to: u8,
    }
    impl Parsable<'_> for Move {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                (
                    preceded(tag("move"), separated(nom::character::complete::u8)),
                    preceded(tag("from"), separated(nom::character::complete::u8)),
                    preceded(tag("to"), trim(false, nom::character::complete::u8)),
                ),
                |(n, from, to)| Self {
                    number: n.into(),
                    from,
                    to,
                },
            )
            .parse(input)
        }
    }

    /// The model of crane being used to move crates, with each part of
    /// the problem using a different crane.
    pub trait Crane {
        /// Executes a move on the `stacks` in place.
        fn execute_move(muv: &Move, stacks: &mut Stacks) -> AocResult<()>;
    }

    /// The CrateMover 9000 crane (for part one), which can only
    /// move crates one at a time.
    pub struct CrateMover9000;
    impl Crane for CrateMover9000 {
        fn execute_move(muv: &Move, stacks: &mut Stacks) -> AocResult<()> {
            for _ in 0..muv.number {
                let cr = stacks.get_stack(muv.from)?.pop().ok_or_else(|| {
                    AocError::Process(
                        format!(
                            "Tried to pop an element from stack {}, but it is empty!",
                            muv.from
                        )
                        .into(),
                    )
                })?;
                stacks.get_stack(muv.to)?.push(cr);
            }

            Ok(())
        }
    }

    /// The CrateMover 9001 crane (for part two), which can move
    /// multiple crates at once.
    pub struct CrateMover9001;
    impl Crane for CrateMover9001 {
        fn execute_move(muv: &Move, stacks: &mut Stacks) -> AocResult<()> {
            let from_stack = stacks.get_stack(muv.from)?;

            if from_stack.len() < muv.number {
                return Err(AocError::Process(
                    format!(
                        "Stack {} does not contain the required {} crates",
                        muv.from, muv.number
                    )
                    .into(),
                ));
            }

            // Get crates from the top of the stack
            let left = from_stack.len() - muv.number;
            let crates = from_stack[left..].to_vec();

            // Remove them from the old stack
            from_stack.truncate(left);

            // Add them to the new stack
            stacks.get_stack(muv.to)?.extend(crates);

            Ok(())
        }
    }

    /// The problem definition, consisting of the crate stacks and list of moves.
    #[derive(Debug)]
    pub struct Problem {
        /// The stacks of crates.
        stacks: Stacks,
        /// The list of moves to execute.
        moves: Vec<Move>,
    }
    impl FromStr for Problem {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let secs = s.sections(2)?;

            Ok(Self {
                stacks: Stacks::from_str(secs[0])?,
                moves: Move::gather(secs[1].lines())?,
            })
        }
    }
    impl Problem {
        /// Executes a list of moves for a particular [`Crane`] and returns
        /// the resulting crate stacks.
        pub fn execute_moves<C: Crane>(&self) -> AocResult<Stacks> {
            let mut stacks = self.stacks.clone();
            for muv in self.moves.iter() {
                C::execute_move(muv, &mut stacks)?;
            }

            Ok(stacks)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 5,
    name: "Supply Stack",
    preprocessor: Some(|input| Ok(Box::new(Problem::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Problem>()?
                .execute_moves::<CrateMover9000>()?
                .top_crates()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Problem>()?
                .execute_moves::<CrateMover9001>()?
                .top_crates()
                .into())
        },
    ],
};
