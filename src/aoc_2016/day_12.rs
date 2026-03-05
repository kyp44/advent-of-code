use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "cpy 41 a
inc a
inc a
dec a
jnz a 2
dec a";
            answers = signed![42];
        }
        example {
            input = "inc c
cpy c a
inc a";
            answers = signed![2, 3];
        }
        actual_answers = signed![318083, 9227737];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::collections::HashMap;

    use super::*;
    use aoc::parse::{field_line_parser, trim};
    use maplit::hashmap;
    use nom::{branch::alt, bytes::tag, character::complete::isize as pisize, combinator::map};

    /// One of the computer registers.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum Register {
        /// Register A.
        A,
        /// Register B.
        B,
        /// Register C.
        C,
        /// Register D.
        D,
    }
    impl Parsable for Register {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            alt((
                map(tag("a"), |_| Self::A),
                map(tag("b"), |_| Self::B),
                map(tag("c"), |_| Self::C),
                map(tag("d"), |_| Self::D),
            ))
            .parse(input)
        }
    }

    /// The computer registers.
    #[derive(Clone, Debug)]
    pub struct Registers(HashMap<Register, isize>);
    impl Registers {
        /// Creates the registers directly from the values.
        pub fn new(a: isize, b: isize, c: isize, d: isize) -> Self {
            Self(hashmap! {
                Register::A => a,
                Register::B => b,
                Register::C => c,
                Register::D => d,
            })
        }
    }
    impl Default for Registers {
        fn default() -> Self {
            Self::new(0, 0, 0, 0)
        }
    }
    impl SimpleRegisters for Registers {
        type Key = Register;
        type Value = isize;

        fn map(&self) -> &std::collections::HashMap<Self::Key, Self::Value> {
            &self.0
        }

        fn map_mut(&mut self) -> &mut std::collections::HashMap<Self::Key, Self::Value> {
            &mut self.0
        }
    }

    /// An operand for some of the [`AsmInstruction`]s.
    #[derive(Debug)]
    pub enum Operand {
        /// The value in one of the registers.
        Register(Register),
        /// An explicit numeric value.
        Number(isize),
    }
    impl Parsable for Operand {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            alt((
                map(pisize, Self::Number),
                map(Register::parser, Self::Register),
            ))
            .parse(input)
        }
    }
    impl Operand {
        /// Returns the value of the operand regardless of which type of operand
        /// it is.
        pub fn value(&self, registers: &Registers) -> isize {
            match self {
                Operand::Register(reg) => *registers.get(reg),
                Operand::Number(n) => *n,
            }
        }
    }

    /// An assembunny instruction.
    #[derive(Debug)]
    pub enum AsmInstruction {
        /// Copy a value into a register.
        Copy(Operand, Register),
        /// Increment the value in a register.
        Increment(Register),
        /// Decrement the value in a register.
        Decrement(Register),
        /// Jump if the value is nonzero.
        JumpNz(Operand, isize),
    }
    impl Parsable for AsmInstruction {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            alt((
                map(
                    field_line_parser("cpy ", (Operand::parser, trim(false, Register::parser))),
                    |(opx, regy)| Self::Copy(opx, regy),
                ),
                map(field_line_parser("inc ", Register::parser), |reg| {
                    Self::Increment(reg)
                }),
                map(field_line_parser("dec ", Register::parser), |reg| {
                    Self::Decrement(reg)
                }),
                map(
                    field_line_parser("jnz ", (Operand::parser, trim(false, pisize))),
                    |(op, jmp)| Self::JumpNz(op, jmp),
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
                AsmInstruction::Copy(op, reg) => {
                    *registers.get_mut(reg) = op.value(registers);

                    None
                }
                AsmInstruction::Increment(reg) => {
                    *registers.get_mut(reg) += 1;

                    None
                }
                AsmInstruction::Decrement(reg) => {
                    *registers.get_mut(reg) -= 1;

                    None
                }
                AsmInstruction::JumpNz(op, d) => {
                    (op.value(registers) != 0).then_some(Jump::Relative(*d))
                }
            }))
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 12,
    name: "Leonardo's Monorail",
    preprocessor: Some(|input| Ok(Box::new(Program::<AsmInstruction>::parse(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(i64::try_from(
                *input
                    .expect_data::<Program<AsmInstruction>>()?
                    .execute(Registers::default())?
                    .registers()
                    .get(&Register::A),
            )
            .unwrap()
            .into())
        },
        // Part two
        |input| {
            // Changed initial registers
            let registers = Registers::new(0, 0, 1, 0);

            Ok(i64::try_from(
                *input
                    .expect_data::<Program<AsmInstruction>>()?
                    .execute(registers)?
                    .registers()
                    .get(&Register::A),
            )
            .unwrap()
            .into())
        },
    ],
};
