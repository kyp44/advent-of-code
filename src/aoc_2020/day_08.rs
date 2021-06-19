use super::super::aoc::{
    AocError,
    Parseable,
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
        vec![5],
        vec![1087]
    }
}

#[derive(Debug)]
enum Instruction {
    Nop,
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
                        "nop" => Instruction::Nop,
                        "acc" => Instruction::Acc(n),
                        "jmp" => Instruction::Jmp(n),
                        _ => panic!(),
                    }
                }
            )
        )(input)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 8,
    name: "Handheld Halting",
    solver: |input| {
        // Generation
        let program = Instruction::gather(input.lines())?;
        
        // Processing
        let mut pc = 0;
        let mut acc = 0;
        let mut executed_pcs: HashSet<usize> = HashSet::new();
        loop {
            if !executed_pcs.insert(pc) {
                break;
            }
            let inst = program.get(pc).unwrap();
            //match program.get(pc).ok_or_else(|| )? {

            // Let instruction affect the program counter and accumulator
            let mut ipc = pc as i32;
            if let Instruction::Jmp(d) = inst {
                ipc += d;
            } else {
                if let Instruction::Acc(d) = inst {
                    acc += d;
                }
                ipc += 1;
            }

            if ipc < 0 || ipc >= program.len() as i32 {
                return Err(AocError::Process(format!("Program has gone outside of its bounds to pc = {}", pc)));
            }
            pc = ipc as usize;
        };

        if acc < 0 {
            return Err(AocError::Process(format!("Accumulator ended up as {}, which is a problem", acc)));
        }

        let answers = vec![acc as u32];
        Ok(answers)
    }
};
