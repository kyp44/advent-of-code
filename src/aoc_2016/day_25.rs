use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "cpy 37 b
dec a
dec b
jnz b -2
jnz a 4
out 0
out 1
jnz 1 -2
out 2
jnz 1 -1
";
            answers = unsigned![37];
        }
        actual_answers = unsigned![175];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use crate::aoc_2016::day_12::solution::{
        AsmInstruction as OriginalInstruction, Operand, Registers,
    };
    use aoc::parse::trim;
    use nom::{branch::alt, bytes::tag, combinator::map};

    /// An assembunny instruction.
    ///
    /// Can be parsed from text input.
    #[derive(Clone, Debug)]
    enum AsmInstruction {
        /// An original instruction from the [`day_12`](crate::aoc_2016::day_12)
        /// problem.
        Original(OriginalInstruction),
        /// Emit a value to the output.
        Output(Operand),
    }
    impl Parsable for AsmInstruction {
        type Parsed<'a> = Self;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            alt((
                map(OriginalInstruction::parser, Self::Original),
                map((tag("out "), trim(false, Operand::parser)), |(_, op)| {
                    Self::Output(op)
                }),
            ))
            .parse(input)
        }
    }
    impl Instruction for AsmInstruction {
        type Registers = Registers;
        type YieldItem = Option<isize>;
        type Err = AocError;

        fn execute(
            &self,
            program_counter: Option<&mut ProgramCounter<Self>>,
            registers: &mut Self::Registers,
        ) -> Result<Self::YieldItem, Self::Err> {
            let program_counter = program_counter.unwrap();

            match self {
                AsmInstruction::Original(inst) => {
                    program_counter.with_dummy(|pc| inst.execute(Some(pc), registers))?;
                    Ok(None)
                }
                AsmInstruction::Output(op) => {
                    program_counter.increment();
                    Ok(Some(op.value(registers)))
                }
            }
        }
    }

    /// A program to drive the clock output.
    pub struct ClockProgram {
        /// The standard program.
        program: Program<AsmInstruction>,
    }
    impl FromStr for ClockProgram {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Program::parse(s).map(|program| Self { program })
        }
    }
    impl ClockProgram {
        /// Executes the program and returns whether or not the desired clock
        /// pattern (`0`, `1`, `0`, `1`, `0`, `1`, ...) is emitted for a
        /// particular input (which is pre-loaded into register `a`).
        fn execute(&self, input: isize) -> AocResult<bool> {
            let executor = self
                .program
                .monitored_executor(Registers::new(input, 0, 0, 0));
            let mut minimal_patern_met = false;
            let mut next_expected = false;

            for exec_res in executor {
                let inst_end = exec_res?;

                if inst_end.repeated_state {
                    return Ok(minimal_patern_met);
                }

                if let Some(next) = inst_end.yielded_item {
                    let next = match next {
                        0 => false,
                        1 => true,
                        _ => return Ok(false),
                    };

                    if next == next_expected {
                        // If we have gotten a zero then a one then any repeat will repeat the
                        // correct pattern forever.
                        if next_expected {
                            minimal_patern_met = true;
                        }
                        next_expected = !next_expected;
                    } else {
                        // The correct pattern has been broken so we are done
                        return Ok(false);
                    }
                }
            }

            Err(AocError::Process("Program terminated!".into()))
        }

        /// Finds the minimum input for which the program outputs the correct
        /// clock signal.
        ///
        /// This is done by executing the program with input `0` and
        /// incrementing the input until it produces the correct clock
        /// signal.
        pub fn find_minimal_good_input(&self) -> AocResult<u64> {
            for input in 0.. {
                if self.execute(input)? {
                    return Ok(u64::try_from(input).unwrap());
                }
            }

            Err(AocError::NoSolution)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 25,
    name: "Clock Signal",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let program = ClockProgram::from_str(input.expect_text()?)?;

            // Process
            Ok(program.find_minimal_good_input()?.into())
        },
    ],
};
