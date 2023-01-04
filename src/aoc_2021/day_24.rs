use std::str::FromStr;

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use Answer::Unsigned;

    #[test]
    #[ignore]
    fn actual() {
        assert_eq!(
            SOLUTION
                .run_and_print(super::super::YEAR_SOLUTIONS.year)
                .unwrap(),
            vec![Unsigned(11120)],
        );
    }

    #[test]
    fn example() {
        let input = "inp w
add z w
mod z 2
div w 2
add y w
mod y 2
div w 2
add x w
mod x 2
div w 2
mod w 2";

        let program = Program::from_str(input).unwrap();

        let check = |n: Number, regs: (Number, Number, Number, Number)| {
            assert_eq!(
                program.execute(&[n]).unwrap(),
                Registers::new(regs.0, regs.1, regs.2, regs.3)
            );
        };

        check(0, (0, 0, 0, 0));
        check(3, (0, 0, 1, 1));
        check(5, (0, 1, 0, 1));
        check(6, (0, 1, 1, 0));
        check(9, (1, 0, 0, 1));
        check(10, (1, 0, 1, 0));
        check(13, (1, 1, 0, 1));
        check(15, (1, 1, 1, 1));
    }
}

mod solution {
    use std::str::FromStr;

    use enum_map::{enum_map, Enum, EnumMap};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{one_of, space1},
        combinator::{map, opt},
        sequence::{pair, preceded},
    };

    use super::*;

    pub type Number = i64;

    #[derive(Debug, Enum, Clone, Copy)]
    pub enum Register {
        W,
        X,
        Y,
        Z,
    }
    impl<'a> Parseable<'a> for Register {
        fn parser(input: &'a str) -> NomParseResult<&str, Self>
        where
            Self: Sized,
        {
            map(one_of("wxyz"), |c| match c {
                'w' => Self::W,
                'x' => Self::X,
                'y' => Self::Y,
                'z' => Self::Z,
                _ => panic!(),
            })(input)
        }
    }

    #[derive(Debug)]
    enum Operand {
        Register(Register),
        Number(Number),
    }
    impl<'a> Parseable<'a> for Operand {
        fn parser(input: &'a str) -> NomParseResult<&str, Self>
        where
            Self: Sized,
        {
            alt((
                map(Register::parser, |r| Self::Register(r)),
                map(nom::character::complete::i64, |n| Self::Number(n)),
            ))(input)
        }
    }

    #[derive(Debug)]
    enum Instruction {
        ReadInput(Register),
        Add(Register, Operand),
        Multiply(Register, Operand),
        Divide(Register, Operand),
        Modulo(Register, Operand),
        Equal(Register, Operand),
    }
    impl<'a> Parseable<'a> for Instruction {
        fn parser(input: &'a str) -> NomParseResult<&str, Self>
        where
            Self: Sized,
        {
            fn operands_parser(input: &str) -> NomParseResult<&str, (Register, Option<Operand>)> {
                preceded(
                    space1,
                    pair(Register::parser, opt(preceded(space1, Operand::parser))),
                )(input)
            }

            alt((
                map(pair(tag("inp"), operands_parser), |(_, op)| {
                    Self::ReadInput(op.0)
                }),
                map(pair(tag("add"), operands_parser), |(_, op)| {
                    Self::Add(op.0, op.1.unwrap())
                }),
                map(pair(tag("mul"), operands_parser), |(_, op)| {
                    Self::Multiply(op.0, op.1.unwrap())
                }),
                map(pair(tag("div"), operands_parser), |(_, op)| {
                    Self::Divide(op.0, op.1.unwrap())
                }),
                map(pair(tag("mod"), operands_parser), |(_, op)| {
                    Self::Modulo(op.0, op.1.unwrap())
                }),
                map(pair(tag("eql"), operands_parser), |(_, op)| {
                    Self::Equal(op.0, op.1.unwrap())
                }),
            ))(input)
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct Registers {
        values: EnumMap<Register, Number>,
    }
    impl Registers {
        pub fn new(w: Number, x: Number, y: Number, z: Number) -> Self {
            Self {
                values: enum_map! {
                    Register::W => w,
                    Register::X => x,
                    Register::Y => y,
                    Register::Z => z,
                },
            }
        }

        pub fn value(&self, reg: Register) -> Number {
            self.values[reg]
        }

        fn operand_value(&self, operand: &Operand) -> Number {
            match operand {
                Operand::Register(reg) => self.values[*reg],
                Operand::Number(n) => *n,
            }
        }

        fn execute(
            &mut self,
            instruction: &Instruction,
            inputs: &mut impl Iterator<Item = Number>,
        ) -> AocResult<()> {
            match instruction {
                Instruction::ReadInput(reg) => {
                    self.values[*reg] = inputs
                        .next()
                        .ok_or(AocError::Process("Ran out of program inputs!".into()))?;
                }
                Instruction::Add(reg, op) => self.values[*reg] += self.operand_value(op),
                Instruction::Multiply(reg, op) => self.values[*reg] *= self.operand_value(op),
                Instruction::Divide(reg, op) => self.values[*reg] /= self.operand_value(op),
                Instruction::Modulo(reg, op) => self.values[*reg] %= self.operand_value(op),
                Instruction::Equal(reg, op) => {
                    self.values[*reg] = (self.values[*reg] == self.operand_value(op)).into()
                }
            }

            Ok(())
        }
    }
    impl Default for Registers {
        fn default() -> Self {
            Self::new(0, 0, 0, 0)
        }
    }

    pub struct Program {
        instructions: Vec<Instruction>,
    }
    impl FromStr for Program {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let instructions = Instruction::gather(s.lines().filter_map(|line| {
                let trimmed = line.trim();

                // Filter out blank and comment lines
                if trimmed.starts_with("#") || trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            }))?;

            Ok(Self { instructions })
        }
    }
    impl Program {
        pub fn execute(&self, inputs: &[Number]) -> AocResult<Registers> {
            let mut registers = Registers::default();
            let mut inputs = inputs.iter().copied();

            // Run every instruction
            for instruction in self.instructions.iter() {
                registers.execute(instruction, &mut inputs)?;
            }

            // Ensure that every input was used
            if inputs.count() > 0 {
                Err(AocError::Process(
                    "Not all inputs were used by the program".into(),
                ))
            } else {
                Ok(registers)
            }
        }
    }
}

use solution::*;

pub const SOLUTION: Solution = Solution {
    day: 24,
    name: "Arithmetic Logic Unit",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let program = Program::from_str(input.expect_input()?)?;

            // Process
            fn split_digits(n: u64) -> Vec<Number> {
                let mut digits = Vec::new();
                let mut n = n;

                while n > 0 {
                    digits.push((n % 10).try_into().unwrap());
                    n /= 10;
                }

                digits.reverse();
                digits
            }

            //for n in (1u64..99999999999999).rev() {
            for n in (11111111111111u64..=99999999999999).rev() {
                let digits = split_digits(n);

                if n % 1000000 == 0 {
                    println!("On {n}");
                }

                if !digits.contains(&0) {
                    if program.execute(&digits)?.value(Register::Z) == 0 {
                        println!("Giggles: {n}");
                        break;
                    }
                }
            }

            /* for n in (99999999999999 - 1000..=99999999999999).rev() {
                let digits = split_digits(n);

                if n % 1000000 == 0 {
                    println!("On {n}");
                }

                if !digits.contains(&0) {
                    let z = program.execute(&digits)?.value(Register::Z);

                    println!("{n}: {z}");
                }
            } */

            /* println!(
                "Result: {:?}",
                program.execute(&[3, 5, 6, 8, 9, 4, 6, 2, 1, 8, 7, 5, 3, 6])?
            ); */

            Ok(Answer::Unsigned(0))
        },
    ],
};
