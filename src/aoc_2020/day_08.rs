use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "nop +0
acc +1
jmp +4
acc +3
jmp -3
acc -99
acc +1
jmp -4
acc +6";
            answers = unsigned![5, 8];
        }
        actual_answers = unsigned![1087, 780];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use nom::{
        branch::alt, bytes::complete::tag, character::complete::space1, combinator::map,
        error::context, sequence::separated_pair,
    };
    use std::convert::TryInto;
    use std::iter::{Enumerate, Filter};
    use std::slice::Iter;

    /// A single program instruction with operand, which can be parsed from text
    /// input.
    #[derive(Debug, Clone)]
    pub enum AsmInstruction {
        /// `nop` instruction, which does nothing.
        Nop(isize),
        /// `acc` instruction, which adds a value to the accumulator register.
        Acc(isize),
        /// `jmp` instruction, which jumps to a relative instruction.
        Jmp(isize),
    }
    impl Parsable for AsmInstruction {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            context(
                "instruction",
                map(
                    separated_pair(
                        alt((tag("nop"), tag("acc"), tag("jmp"))),
                        space1,
                        nom::character::complete::isize,
                    ),
                    |(iss, n)| match iss {
                        "nop" => AsmInstruction::Nop(n),
                        "acc" => AsmInstruction::Acc(n),
                        "jmp" => AsmInstruction::Jmp(n),
                        _ => panic!(),
                    },
                ),
            )
            .parse(input)
        }
    }
    impl Instruction for AsmInstruction {
        type Registers = AccumulatorRegister;
        type YieldItem = ();
        type Error = AocError;

        fn execute(
            &self,
            registers: &mut Self::Registers,
        ) -> Result<Executed<Self::YieldItem>, Self::Error> {
            Ok(Executed::only_jump(match self {
                AsmInstruction::Nop(_) => None,
                AsmInstruction::Acc(n) => {
                    registers.value += *n;
                    None
                }
                AsmInstruction::Jmp(d) => Some(Jump::Relative(*d)),
            }))
        }
    }

    /// The accumulator register.
    #[derive(Clone, Copy, Default, Debug)]
    pub struct AccumulatorRegister {
        /// The current value.
        value: isize,
    }
    impl AccumulatorRegister {
        /// Verifies that the register is positive and converts it.
        pub fn verify_positive(&self) -> AocResult<u32> {
            if self.value < 0 {
                return Err(AocError::Process(
                    format!(
                        "Accumulator ended up negative as {}, which is a problem",
                        self.value
                    )
                    .into(),
                ));
            }
            Ok(self.value.try_into().unwrap())
        }
    }
    // These need to always be equal to each other since comparison is just used for
    // compare program states and we just want equivalent states to just look at the
    // program counter.
    impl PartialEq for AccumulatorRegister {
        fn eq(&self, _other: &Self) -> bool {
            true
        }
    }
    impl Eq for AccumulatorRegister {}
    impl std::hash::Hash for AccumulatorRegister {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            0.hash(state);
        }
    }

    /// Returns a [`ProgramVariations`] iterator over variations on the
    /// program.
    pub fn variations(original: &Program<AsmInstruction>) -> ProgramVariations<'_> {
        ProgramVariations {
            original,
            iter: original
                .instructions()
                .iter()
                .enumerate()
                .filter(|(_, inst)| {
                    matches!(inst, AsmInstruction::Nop(_) | AsmInstruction::Jmp(_))
                }),
        }
    }

    /// Type of the filter [`Iterator`] used by the [`ProgramVariations`]
    /// iterator.
    type VariationsIterator<'a> =
        Filter<Enumerate<Iter<'a, AsmInstruction>>, fn(&(usize, &AsmInstruction)) -> bool>;

    /// [`Iterator`] over variations of a program  with every `jmp` instruction
    /// replaced with a `nop` instruction and vice versa.
    pub struct ProgramVariations<'a> {
        /// Original program that is being varied.
        original: &'a Program<AsmInstruction>,
        /// [`Iterator`] over the `jmp` and `nop` instructions in the program.
        iter: VariationsIterator<'a>,
    }
    impl Iterator for ProgramVariations<'_> {
        type Item = Program<AsmInstruction>;

        fn next(&mut self) -> Option<Self::Item> {
            // Look for the next NOP or JMP instruction
            self.iter.next().map(|(pc, inst)| {
                use AsmInstruction::*;

                let mut new_instructions = self.original.instructions().to_vec();

                new_instructions[pc] = match inst {
                    Nop(v) => Jmp(*v),
                    Jmp(v) => Nop(*v),
                    _ => panic!(),
                };

                Program::new(new_instructions)
            })
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 8,
    name: "Handheld Halting",
    preprocessor: Some(|input| Ok(Box::new(Program::<AsmInstruction>::parse(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Processing
            let end = input
                .expect_data::<Program<AsmInstruction>>()?
                .execute_monitored(AccumulatorRegister::default())?;

            Ok(Answer::Unsigned(
                match end.end_status {
                    ProgramEndStatus::Infinite => end.registers().verify_positive()?,
                    _ => {
                        return Err(AocError::Process(
                            "Program execution did not result in an infinite loop".into(),
                        ));
                    }
                }
                .into(),
            ))
        },
        // Part two
        |input| {
            // Processing
            let mut terminated_acc = None;
            for prog in variations(input.expect_data::<Program<AsmInstruction>>()?) {
                let end = prog.execute_monitored(AccumulatorRegister::default())?;

                if !matches!(end.end_status, ProgramEndStatus::Infinite) {
                    terminated_acc = Some(end.registers().verify_positive()?);
                    break;
                }
            }
            Ok(Answer::Unsigned(
                terminated_acc
                    .ok_or_else(|| AocError::Process("No modified programs terminated!".into()))?
                    .into(),
            ))
        },
    ],
};
