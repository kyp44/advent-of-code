use super::super::aoc::{
    AocError,
    Parseable,
    ParseError,
    ParseResult,
    Solution,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, space1, one_of},
    combinator::map,
    error::context,
    sequence::{separated_pair, pair},
};
use std::collections::HashSet;
use std::str::FromStr;
use std::iter::Enumerate;
use std::iter::Filter;
use std::slice::Iter;

#[cfg(test)]
mod tests{
    use super::*;
    use crate::solution_test;
    
    solution_test! {
        "nop +0
acc +1
jmp +4
acc +3
jmp -3
acc -99
acc +1
jmp -4
acc +6",
        vec![5, 8],
        vec![1087, 780]
    }
}

#[derive(Debug, Clone)]
enum Instruction {
    Nop(i32),
    Acc(i32),
    Jmp(i32),
}

impl Parseable for Instruction {
    fn parse(input: &str) -> ParseResult<Self> {
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
                }
            )
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
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Program {
            instructions: Instruction::gather(s.lines())?
        })
    }
}

impl Program {
    fn variations(&self) -> ProgramVariations {
        ProgramVariations {
            original: self,
            iter: self.instructions.iter()
                .enumerate().filter(|(_, inst)| {
                    matches!(inst, Instruction::Nop(_) | Instruction::Jmp(_))
                }),
        }
    }
}

type VariationsIterator<'a> = Filter<Enumerate<Iter<'a, Instruction>>, fn(&(usize, &Instruction)) -> bool>;
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
            let mut ipc = pc as i32;
            if let Instruction::Jmp(d) = inst {
                ipc += d;
                if ipc < 0 || ipc > self.instructions.len() as i32 {
                    break ProgramEndStatus::JumpedOut(acc);
                }
            } else {
                if let Instruction::Acc(d) = inst {
                    acc += d;
                }
                ipc += 1;
            }

            pc = ipc as usize;
            if pc == self.instructions.len() {
                break ProgramEndStatus::Terminated(acc);
            }
        }
    }
}

pub const SOLUTION: Solution = Solution {
    day: 8,
    name: "Handheld Halting",
    solver: |input| {
        // Generation
        let program: Program = input.parse()?;
        
        // Processing
        fn check_acc(acc: i32) -> Result<u32, AocError> {
            if acc < 0 {
                return Err(AocError::Process(format!("Accumulator ended up negative as {}, which is a problem", acc)))
            }
            Ok(acc as u32)
        }
        
        let part_a = match program.execute() {
            ProgramEndStatus::Infinite(acc) => check_acc(acc)?,
            _ => {
                return Err(AocError::Process("Program execution did not result in an infinite loop".to_string()));
            }
        }.into();
        let mut part_b = None;
        for prog in program.variations() {
            if let ProgramEndStatus::Terminated(acc) = prog.execute() {
                part_b = Some(check_acc(acc)?);
                break;
            }
        }
        let part_b = part_b.ok_or_else(|| AocError::Process("No modified programs terminated!".to_string()))?.into();
        
        Ok(vec![part_a, part_b])
    }
};
