use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "inc a
inc b
tpl b
jio a, +2
tpl b
inc b";
            answers = unsigned![4, 10];
        }
        actual_answers = unsigned![170, 247];
    }
}

/// Contains solution implementation items.
mod solution {
    use aoc::parse::{field_line_parser, trim};
    use num::Integer;

    use super::*;
    use maplit::hashmap;
    use nom::{branch::alt, bytes::complete::tag, combinator::map, sequence::separated_pair};
    use std::collections::HashMap;

    /// One of the computer's registers, which can be parsed from text input.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum Register {
        /// Register `a`.
        A,
        /// Register `b`.
        B,
    }
    impl Parsable for Register {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            alt((
                map(tag("a"), |_| Register::A),
                map(tag("b"), |_| Register::B),
            ))
            .parse(input)
        }
    }

    /// The state of the computer's registers.
    #[derive(Clone)]
    pub struct Registers(HashMap<Register, u64>);
    impl Registers {
        /// Creates the registers directly from their values.
        pub fn new(a: u64, b: u64) -> Self {
            Self(hashmap! { Register::A => a, Register::B => b })
        }
    }
    impl SimpleRegisters for Registers {
        type Key = Register;
        type Value = u64;

        fn map(&self) -> &HashMap<Self::Key, Self::Value> {
            &self.0
        }

        fn map_mut(&mut self) -> &mut HashMap<Self::Key, Self::Value> {
            &mut self.0
        }
    }

    /// Possible instructions of the computer, which can be parsed from text
    /// input.
    #[derive(Debug)]
    pub enum AsmInstruction {
        /// The `hlf` instruction operating on register.
        Half(Register),
        /// The `tpl` instruction operating on a register.
        Triple(Register),
        /// The `inc` instruction operating on a register.
        Increment(Register),
        /// The `jmp` instruction with the relative offset.
        Jump(isize),
        /// The conditional `jie` instruction with the register to check and
        /// relative offset.
        JumpIfEven(Register, isize),
        /// The conditional `jio` instruction with the register to check and
        /// relative offset.
        JumpIfOne(Register, isize),
    }
    impl Parsable for AsmInstruction {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            use nom::character::complete::isize as pisize;

            alt((
                map(field_line_parser("hlf ", Register::parser), |r| {
                    AsmInstruction::Half(r)
                }),
                map(field_line_parser("tpl ", Register::parser), |r| {
                    AsmInstruction::Triple(r)
                }),
                map(field_line_parser("inc ", Register::parser), |r| {
                    AsmInstruction::Increment(r)
                }),
                map(field_line_parser("jmp ", pisize), AsmInstruction::Jump),
                map(
                    field_line_parser(
                        "jie ",
                        separated_pair(
                            trim(false, Register::parser),
                            tag(","),
                            trim(false, pisize),
                        ),
                    ),
                    |(r, o)| AsmInstruction::JumpIfEven(r, o),
                ),
                map(
                    field_line_parser(
                        "jio ",
                        separated_pair(
                            trim(false, Register::parser),
                            tag(","),
                            trim(false, pisize),
                        ),
                    ),
                    |(r, o)| AsmInstruction::JumpIfOne(r, o),
                ),
            ))
            .parse(input)
        }
    }
    impl Instruction for AsmInstruction {
        type Registers = Registers;
        type YieldItem = ();
        type Err = AocError;

        fn execute(
            &self,
            registers: &mut Self::Registers,
        ) -> Result<Executed<Self::YieldItem>, Self::Err> {
            Ok(Executed::only_jump(match self {
                AsmInstruction::Half(r) => {
                    registers.modify(*r, |r| r / 2);
                    None
                }
                AsmInstruction::Triple(r) => {
                    registers.modify(*r, |r| 3 * r);
                    None
                }
                AsmInstruction::Increment(r) => {
                    *registers.get_mut(r) += 1;
                    None
                }
                AsmInstruction::Jump(o) => Some(Jump::Relative(*o)),
                AsmInstruction::JumpIfEven(r, o) => {
                    registers.get(r).is_even().then_some(Jump::Relative(*o))
                }

                AsmInstruction::JumpIfOne(r, o) => {
                    (*registers.get(r) == 1).then_some(Jump::Relative(*o))
                }
            }))
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 23,
    name: "Opening the Turing Lock",
    preprocessor: Some(|input| Ok(Box::new(Program::<AsmInstruction>::parse(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok((*input
                .expect_data::<Program<AsmInstruction>>()?
                .execute(Registers::new(0, 0))?
                .registers()
                .get(&Register::B))
            .into())
        },
        // Part two
        |input| {
            // Process
            Ok((*input
                .expect_data::<Program<AsmInstruction>>()?
                .execute(Registers::new(1, 0))?
                .registers()
                .get(&Register::B))
            .into())
        },
    ],
};
