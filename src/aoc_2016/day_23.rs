use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "cpy 2 a
tgl a
tgl a
tgl a
cpy 1 a
dec a
dec a";
            answers = signed![3];
        }
        example {
            input = "cpy 2 b
tgl b
tgl b
nop
tgl a
mul a a d
inc d
inc d
cpy d a";
            answers = signed![66, 171];
        }
        actual_answers = signed![11123, 479007683];
    }
}

/// Contains solution implementation items.
///
/// This problem is not great because part two relies on recognizing patterns in
/// the specific program to optimize by replacing some slow sequences of
/// instructions with a single, much faster multiplication instruction.
mod solution {
    use super::*;
    use crate::aoc_2016::day_12::solution::{
        AsmInstruction as OriginalInstruction, Operand, Register, Registers,
    };
    use aoc::parse::trim;
    use nom::{branch::alt, bytes::complete::tag, combinator::map, sequence::preceded};

    /// An assembunny instruction.
    ///
    /// Can be parsed from text input.
    #[derive(Clone, Debug)]
    enum AsmInstruction {
        /// Does nothing.
        Nop,
        /// An original instruction from the [`day_12`](crate::aoc_2016::day_12)
        /// problem.
        Original(OriginalInstruction),
        /// Multiply two operands together and store the result in a register.
        Multiply(Operand, Operand, Register),
        /// Toggles the instruction at the relative distance (from this
        /// instruction) stored in the register per the rules.
        Toggle(Register),
    }
    impl Parsable for AsmInstruction {
        type Parsed<'a> = Self;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            alt((
                map(tag("nop"), |_| Self::Nop),
                map(OriginalInstruction::parser, Self::Original),
                map(
                    (
                        tag("mul "),
                        trim(false, Operand::parser),
                        trim(false, Operand::parser),
                        trim(false, Register::parser),
                    ),
                    |(_, a, b, c)| Self::Multiply(a, b, c),
                ),
                map(preceded(tag("tgl "), Register::parser), Self::Toggle),
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
            program_counter: Option<&mut ProgramCounter<Self>>,
            registers: &mut Self::Registers,
        ) -> Result<Self::YieldItem, Self::Err> {
            let program_counter = program_counter.unwrap();

            match self {
                AsmInstruction::Nop => {
                    // Do nothing
                    program_counter.increment();
                }
                AsmInstruction::Original(inst) => {
                    program_counter.with_dummy(|pc| inst.execute(Some(pc), registers))?;
                }
                AsmInstruction::Multiply(a_op, b_op, reg) => {
                    *registers.get_mut(reg) = a_op.value(registers) * b_op.value(registers);
                    program_counter.increment();
                }
                AsmInstruction::Toggle(reg) => {
                    let offset = *registers.get(reg);
                    if let Some(index) = program_counter.relative_index(offset) {
                        program_counter.change_instructions(|instructions| {
                            let inst = &mut instructions[index];

                            match inst {
                                AsmInstruction::Original(orig_inst) => {
                                    *orig_inst = match orig_inst {
                                        OriginalInstruction::Copy(from_op, to_op) => {
                                            OriginalInstruction::JumpNz(*from_op, *to_op)
                                        }
                                        OriginalInstruction::Increment(reg) => {
                                            OriginalInstruction::Decrement(*reg)
                                        }
                                        OriginalInstruction::Decrement(reg) => {
                                            OriginalInstruction::Increment(*reg)
                                        }
                                        OriginalInstruction::JumpNz(test_op, offset_op) => {
                                            OriginalInstruction::Copy(*test_op, *offset_op)
                                        }
                                    }
                                }
                                AsmInstruction::Toggle(reg) => {
                                    *inst = AsmInstruction::Original(
                                        OriginalInstruction::Increment(*reg),
                                    )
                                }
                                _ => {
                                    // The rest of the instructions are their
                                    // own inverses
                                }
                            }
                        });
                    }

                    program_counter.increment();
                }
            }

            Ok(())
        }
    }

    /// A program to emulate the keypad and provide the correct output.
    ///
    /// Can be parsed from text input.
    pub struct KeypadProgram(Program<AsmInstruction>);
    impl FromStr for KeypadProgram {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Program::parse(s).map(Self)
        }
    }
    impl KeypadProgram {
        /// Executes the program and return the value in the A register.
        pub fn execute(&self, initial_a: isize) -> AocResult<i64> {
            self.0
                .execute(Registers::new(initial_a, 0, 0, 0))
                .map(|end| i64::try_from(*end.registers.get(&Register::A)).unwrap())
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 23,
    name: "Safe Cracking",
    preprocessor: Some(|input| Ok(Box::new(KeypadProgram::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<KeypadProgram>()?.execute(7)?.into())
        },
        // Part two
        |input| {
            // Process
            Ok(input.expect_data::<KeypadProgram>()?.execute(12)?.into())
        },
    ],
};
