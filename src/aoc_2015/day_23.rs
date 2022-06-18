use maplit::hashmap;
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    sequence::{preceded, separated_pair},
};
use std::{collections::HashMap, str::FromStr};

use crate::aoc::{parse::trim, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(170), Unsigned(247)],
    "inc a
inc b
tpl b
jio a, +2
tpl b
inc b",
    vec![4u64, 10].answer_vec()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Register {
    A,
    B,
}
impl Parseable<'_> for Register {
    fn parser(input: &str) -> NomParseResult<&str, Self>
    where
        Self: Sized,
    {
        alt((
            map(tag("a"), |_| Register::A),
            map(tag("b"), |_| Register::B),
        ))(input)
    }
}

#[derive(Debug)]
enum Instruction {
    Half(Register),
    Triple(Register),
    Increment(Register),
    Jump(i32),
    JumpIfEven(Register, i32),
    JumpIfOne(Register, i32),
}
impl Parseable<'_> for Instruction {
    fn parser(input: &str) -> NomParseResult<&str, Self>
    where
        Self: Sized,
    {
        alt((
            map(preceded(tag("hlf "), trim(false, Register::parser)), |r| {
                Instruction::Half(r)
            }),
            map(preceded(tag("tpl "), trim(false, Register::parser)), |r| {
                Instruction::Triple(r)
            }),
            map(preceded(tag("inc "), trim(false, Register::parser)), |r| {
                Instruction::Increment(r)
            }),
            map(
                preceded(tag("jmp "), trim(false, nom::character::complete::i32)),
                Instruction::Jump,
            ),
            map(
                preceded(
                    tag("jie "),
                    separated_pair(
                        trim(false, Register::parser),
                        tag(","),
                        trim(false, nom::character::complete::i32),
                    ),
                ),
                |(r, o)| Instruction::JumpIfEven(r, o),
            ),
            map(
                preceded(
                    tag("jio "),
                    separated_pair(
                        trim(false, Register::parser),
                        tag(","),
                        trim(false, nom::character::complete::i32),
                    ),
                ),
                |(r, o)| Instruction::JumpIfOne(r, o),
            ),
        ))(input)
    }
}
impl Instruction {
    fn execute(&self, state: &mut State) {
        let mut register = |r: &Register, f: Box<dyn FnOnce(u64) -> u64>| {
            let reg = state.registers.get_mut(r).unwrap();
            *reg = f(*reg);
            state.program_counter += 1;
        };

        match self {
            Instruction::Half(r) => register(r, Box::new(|r| r / 2)),
            Instruction::Triple(r) => register(r, Box::new(|r| 3 * r)),
            Instruction::Increment(r) => register(r, Box::new(|r| r + 1)),
            Instruction::Jump(o) => state.program_counter += o,
            Instruction::JumpIfEven(r, o) => {
                if state.registers[r] % 2 == 0 {
                    state.program_counter += o;
                } else {
                    state.program_counter += 1;
                }
            }
            Instruction::JumpIfOne(r, o) => {
                if state.registers[r] == 1 {
                    state.program_counter += o
                } else {
                    state.program_counter += 1;
                }
            }
        }
    }
}

#[derive(Debug)]
struct State {
    program_counter: i32,
    registers: HashMap<Register, u64>,
}
impl State {
    fn new(a: u64, b: u64) -> Self {
        State {
            program_counter: 0,
            registers: hashmap! { Register::A => a, Register::B => b },
        }
    }
}

struct Program {
    instructions: Vec<Instruction>,
}
impl FromStr for Program {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Program {
            instructions: Instruction::gather(s.lines())?,
        })
    }
}
impl Program {
    fn execute(&self, mut state: State) -> State {
        loop {
            if state.program_counter < 0 {
                break;
            }
            let pc: usize = state.program_counter.try_into().unwrap();
            if pc >= self.instructions.len() {
                break;
            }
            //println!("Executing: {:?}", self.instructions[pc]);
            self.instructions[pc].execute(&mut state);
            //println!("State {:?}", state);
        }

        state
    }
}

pub const SOLUTION: Solution = Solution {
    day: 23,
    name: "Opening the Turing Lock",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let program: Program = input.expect_input()?.parse()?;

            /*for inst in program.instructions {
                println!("{:?}", inst);
            }*/

            // Process
            let end_state = program.execute(State::new(0, 0));
            Ok(end_state.registers[&Register::B].into())
        },
        // Part b)
        |input| {
            // Generation
            let program: Program = input.expect_input()?.parse()?;

            // Process
            let end_state = program.execute(State::new(1, 0));
            Ok(end_state.registers[&Register::B].into())
        },
    ],
};
