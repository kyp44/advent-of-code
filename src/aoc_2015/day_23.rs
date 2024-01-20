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
    use aoc::parse::trim;
    use num::Integer;

    use super::*;
    use maplit::hashmap;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::map,
        sequence::{preceded, separated_pair},
    };
    use std::{collections::HashMap, str::FromStr};

    /// One of the computer's registers, which can be parsed from text input.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum Register {
        /// Register `a`.
        A,
        /// Register `b`.
        B,
    }
    impl Parsable<'_> for Register {
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

    /// Possible instructions of the computer, which can be parsed from text input.
    #[derive(Debug)]
    enum Instruction {
        /// The `hlf` instruction operating on register.
        Half(Register),
        /// The `tpl` instruction operating on a register.
        Triple(Register),
        /// The `inc` instruction operating on a register.
        Increment(Register),
        /// The `jmp`  instruction with the relative offset.
        Jump(i32),
        /// The conditional `jie` instruction with the register to check and relative offset.
        JumpIfEven(Register, i32),
        /// The conditional `jio` instruction with the register to check and relative offset.
        JumpIfOne(Register, i32),
    }
    impl Parsable<'_> for Instruction {
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
        /// Executes the instruction by modifying the program state.
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
                    if state.registers[r].is_even() {
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

    /// Represents the current state of the computer/program.
    #[derive(Debug)]
    pub struct State {
        /// Current instruction number.
        program_counter: i32,
        /// Current register values.
        pub registers: HashMap<Register, u64>,
    }
    impl State {
        /// Creates a state with given register values.
        pub fn new(a: u64, b: u64) -> Self {
            State {
                program_counter: 0,
                registers: hashmap! { Register::A => a, Register::B => b },
            }
        }
    }

    /// A computer program, which can be parsed from text input.
    pub struct Program {
        /// List of instructions that the program comprises.
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
        /// Executes the program/instructions given a starting state, returning
        /// the final state after completion.
        pub fn execute(&self, mut state: State) -> State {
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
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 23,
    name: "Opening the Turing Lock",
    preprocessor: Some(|input| Ok(Box::new(input.parse::<Program>()?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            let end_state = input.expect_data::<Program>()?.execute(State::new(0, 0));
            Ok(end_state.registers[&Register::B].into())
        },
        // Part two
        |input| {
            // Process
            let end_state = input.expect_data::<Program>()?.execute(State::new(1, 0));
            Ok(end_state.registers[&Register::B].into())
        },
    ],
};
