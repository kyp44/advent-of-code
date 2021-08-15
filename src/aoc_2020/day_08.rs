use crate::aoc::prelude::*;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, one_of, space1},
    combinator::map,
    error::context,
    sequence::{pair, separated_pair},
};
use std::iter::{Enumerate, Filter};
use std::slice::Iter;
use std::str::FromStr;
use std::{collections::HashSet, convert::TryInto};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(1087), Unsigned(780)],
        "nop +0
acc +1
jmp +4
acc +3
jmp -3
acc -99
acc +1
jmp -4
acc +6",
        vec![5u64, 8].answer_vec()
    }
}

#[derive(Debug, Clone)]
enum Instruction {
    Nop(i32),
    Acc(i32),
    Jmp(i32),
}

impl Parseable<'_> for Instruction {
    fn parser(input: &str) -> NomParseResult<Self> {
        context(
            "instruction",
            map(
                separated_pair(
                    alt((tag("nop"), tag("acc"), tag("jmp"))),
                    space1,
                    pair(one_of("+-"), digit1),
                ),
                |(iss, (pm, ns)): (&str, (char, &str))| {
                    let n: i32 = match pm {
                        '-' => -ns.parse::<i32>().unwrap(),
                        '+' => ns.parse().unwrap(),
                        _ => panic!(),
                    };
                    match iss {
                        "nop" => Instruction::Nop(n),
                        "acc" => Instruction::Acc(n),
                        "jmp" => Instruction::Jmp(n),
                        _ => panic!(),
                    }
                },
            ),
        )(input)
    }
}

#[derive(Debug)]
enum ProgramEndStatus {
    JumpedOut(i32),
    Terminated(i32),
    Infinite(i32),
}

#[derive(Debug, Clone)]
struct Program {
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
    fn variations(&self) -> ProgramVariations {
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

type VariationsIterator<'a> =
    Filter<Enumerate<Iter<'a, Instruction>>, fn(&(usize, &Instruction)) -> bool>;
struct ProgramVariations<'a> {
    original: &'a Program,
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

impl Program {
    fn execute(&self) -> ProgramEndStatus {
        let mut pc = 0;
        let mut acc = 0;
        let mut executed_pcs: HashSet<usize> = HashSet::new();
        loop {
            // Insert pc
            if !executed_pcs.insert(pc) {
                // We previouslty executed this instruction, hence and infinite loop
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
                if let Instruction::Acc(d) = inst {
                    acc += d;
                }
                ipc += 1;
            }

            pc = ipc.try_into().unwrap();
            if pc == self.instructions.len() {
                break ProgramEndStatus::Terminated(acc);
            }
        }
    }
}

fn check_acc(acc: i32) -> AocResult<u32> {
    if acc < 0 {
        return Err(AocError::Process(
            format!(
                "Accumulator ended up negative as {}, which is a problem",
                acc
            )
            .into(),
        ));
    }
    Ok(acc.try_into().unwrap())
}

pub const SOLUTION: Solution = Solution {
    day: 8,
    name: "Handheld Halting",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let program: Program = input.parse()?;

            // Processing
            Ok(Answer::Unsigned(
                match program.execute() {
                    ProgramEndStatus::Infinite(acc) => check_acc(acc)?,
                    _ => {
                        return Err(AocError::Process(
                            "Program execution did not result in an infinite loop".into(),
                        ));
                    }
                }
                .into(),
            ))
        },
        // Part b)
        |input| {
            // Generation
            let program: Program = input.parse()?;

            // Processing
            let mut terminated_acc = None;
            for prog in program.variations() {
                if let ProgramEndStatus::Terminated(acc) = prog.execute() {
                    terminated_acc = Some(check_acc(acc)?);
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
