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
    use std::iter::{Enumerate, Filter};
    use std::slice::Iter;
    use std::str::FromStr;
    use std::{collections::HashSet, convert::TryInto};

    /// A single program instruction with operand, which can be parsed from text input.
    #[derive(Debug, Clone)]
    enum Instruction {
        /// `nop` instruction, which does nothing.
        Nop(i32),
        /// `acc` instruction, which adds a value to the accumulator register.
        Acc(i32),
        /// `jmp` instruction, which jumps to a relative instruction.
        Jmp(i32),
    }
    impl Parseable<'_> for Instruction {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            context(
                "instruction",
                map(
                    separated_pair(
                        alt((tag("nop"), tag("acc"), tag("jmp"))),
                        space1,
                        nom::character::complete::i32,
                    ),
                    |(iss, n)| match iss {
                        "nop" => Instruction::Nop(n),
                        "acc" => Instruction::Acc(n),
                        "jmp" => Instruction::Jmp(n),
                        _ => panic!(),
                    },
                ),
            )(input)
        }
    }

    /// The accumulator register.
    #[derive(Default, Debug)]
    pub struct AccumulatorRegister {
        /// The current value.
        value: i32,
    }
    impl AccumulatorRegister {
        /// Applies an instruction to the register, which may have no effect.
        fn apply(&mut self, instruction: &Instruction) {
            if let Instruction::Acc(v) = instruction {
                self.value += v;
            }
        }

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

    /// Possible ways for a program to end, with the value of the accumulator register at this point.
    #[derive(Debug)]
    pub enum ProgramEndStatus {
        /// Jumped outside the bounds of the program instructions.
        JumpedOut(AccumulatorRegister),
        /// Terminated normally.
        Terminated(AccumulatorRegister),
        /// In an infinite loop.
        Infinite(AccumulatorRegister),
    }

    /// A complete program, which can be parsed from text input.
    #[derive(Debug, Clone)]
    pub struct Program {
        /// The ordered list of instructions.
        instructions: Vec<Instruction>,
    }
    impl FromStr for Program {
        type Err = NomParseError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Program {
                instructions: Instruction::gather(s.lines())?,
            })
        }
    }
    impl Program {
        /// Executes the program, returning the end status.
        pub fn execute(&self) -> ProgramEndStatus {
            let mut pc = 0;
            let mut acc = AccumulatorRegister::default();
            let mut executed_pcs: HashSet<usize> = HashSet::new();
            loop {
                // Insert pc
                if !executed_pcs.insert(pc) {
                    // We previously executed this instruction, hence and infinite loop
                    break ProgramEndStatus::Infinite(acc);
                }
                // Fetch the next instruction
                let inst = self.instructions.get(pc).unwrap();

                // Let instruction affect the program counter and accumulator
                let mut ipc: i32 = pc.try_into().unwrap();
                if let Instruction::Jmp(d) = inst {
                    ipc += d;
                    if ipc < 0 || ipc > self.instructions.len().try_into().unwrap() {
                        break ProgramEndStatus::JumpedOut(acc);
                    }
                } else {
                    acc.apply(inst);
                    ipc += 1;
                }

                pc = ipc.try_into().unwrap();
                if pc == self.instructions.len() {
                    break ProgramEndStatus::Terminated(acc);
                }
            }
        }

        /// Returns a [`ProgramVariations`] iterator over variations on the program.
        pub fn variations(&self) -> ProgramVariations {
            ProgramVariations {
                original: self,
                iter: self
                    .instructions
                    .iter()
                    .enumerate()
                    .filter(|(_, inst)| matches!(inst, Instruction::Nop(_) | Instruction::Jmp(_))),
            }
        }
    }

    /// Type of the filter [`Iterator`] used by the [`ProgramVariations`] iterator.
    type VariationsIterator<'a> =
        Filter<Enumerate<Iter<'a, Instruction>>, fn(&(usize, &Instruction)) -> bool>;

    /// [`Iterator`] over variations of a program  with every `jmp` instruction
    /// replaced with a `nop` instruction and vice versa.
    pub struct ProgramVariations<'a> {
        /// Original program that is being varied.
        original: &'a Program,
        /// [`Iterator`] over the `jmp` and `nop` instructions in the program.
        iter: VariationsIterator<'a>,
    }
    impl Iterator for ProgramVariations<'_> {
        type Item = Program;

        fn next(&mut self) -> Option<Self::Item> {
            // Look for the next NOP or JMP instruction
            self.iter.next().map(|(pc, inst)| {
                use Instruction::*;
                let mut new_program = (*self.original).clone();

                new_program.instructions[pc] = match inst {
                    Nop(v) => Jmp(*v),
                    Jmp(v) => Nop(*v),
                    _ => panic!(),
                };

                new_program
            })
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 8,
    name: "Handheld Halting",
    preprocessor: Some(|input| Ok(Box::new(input.parse::<Program>()?).into())),
    solvers: &[
        // Part one
        |input| {
            // Processing
            Ok(Answer::Unsigned(
                match input.expect_data::<Program>()?.execute() {
                    ProgramEndStatus::Infinite(acc) => acc.verify_positive()?,
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
            for prog in input.expect_data::<Program>()?.variations() {
                if let ProgramEndStatus::Terminated(acc) = prog.execute() {
                    terminated_acc = Some(acc.verify_positive()?);
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
