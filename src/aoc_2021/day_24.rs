use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::prelude_test::*;

    #[test]
    #[ignore]
    fn actual() {
        assert_eq!(
            &SOLUTION
                .run_and_print(super::super::YEAR_SOLUTIONS.year)
                .unwrap(),
            unsigned![92967699949891, 91411143612181]
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

/// Contains solution implementation items.
mod solution {
    use enum_map::{Enum, EnumMap, enum_map};
    use itertools::Itertools;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{one_of, space1},
        combinator::{map, opt},
        sequence::{pair, preceded},
    };
    use std::str::FromStr;

    use super::*;

    /// Type to use for ALU numbers.
    pub type Number = i64;

    /// Represents one of the ALU registers, which can be parsed from text input.
    #[derive(Debug, Enum, Clone, Copy)]
    pub enum Register {
        /// The `w` register.
        W,
        /// The `x` register.
        X,
        /// The `y` register.
        Y,
        /// The `z` register.
        Z,
    }
    impl<'a> Parsable<'a> for Register {
        fn parser(input: &'a str) -> NomParseResult<&'a str, Self>
        where
            Self: Sized,
        {
            map(one_of("wxyz"), |c| match c {
                'w' => Self::W,
                'x' => Self::X,
                'y' => Self::Y,
                'z' => Self::Z,
                _ => panic!(),
            })
            .parse(input)
        }
    }

    /// Represents an operand.
    #[derive(Debug)]
    enum Operand {
        /// A register.
        Register(Register),
        /// A number literal.
        Number(Number),
    }
    impl<'a> Parsable<'a> for Operand {
        fn parser(input: &'a str) -> NomParseResult<&'a str, Self>
        where
            Self: Sized,
        {
            alt((
                map(Register::parser, Self::Register),
                map(nom::character::complete::i64, Self::Number),
            ))
            .parse(input)
        }
    }

    /// Represents a single ALU instruction.
    #[derive(Debug)]
    enum Instruction {
        /// `inp` read input instruction.
        ReadInput(Register),
        /// `add` addition instruction.
        Add(Register, Operand),
        /// `mul` multiplication instruction.
        Multiply(Register, Operand),
        /// `div` truncated division instruction.
        Divide(Register, Operand),
        /// `mod` modulo instruction.
        Modulo(Register, Operand),
        /// `equ` equality test instruction.
        Equal(Register, Operand),
    }
    impl<'a> Parsable<'a> for Instruction {
        fn parser(input: &'a str) -> NomParseResult<&'a str, Self>
        where
            Self: Sized,
        {
            /// This is an internal function of [`Instruction::parser`].
            ///
            /// [`nom`] parser to parse a pair of operands, or rather a [`Register`] then an [`Operand`].
            fn operands_parser(input: &str) -> NomParseResult<&str, (Register, Option<Operand>)> {
                preceded(
                    space1,
                    pair(Register::parser, opt(preceded(space1, Operand::parser))),
                )
                .parse(input)
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
            ))
            .parse(input)
        }
    }

    /// A set of registers on which instructions can operate.
    #[derive(Debug, PartialEq, Eq)]
    pub struct Registers {
        /// Values stored in each register.
        values: EnumMap<Register, Number>,
    }
    impl Registers {
        /// Creates a new set of registers with the specified values.
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

        /// Gets the value in a register.
        pub fn value(&self, reg: Register) -> Number {
            self.values[reg]
        }

        /// Gets the value of an operand.
        fn operand_value(&self, operand: &Operand) -> Number {
            match operand {
                Operand::Register(reg) => self.values[*reg],
                Operand::Number(n) => *n,
            }
        }

        /// Executes a single instruction, affecting the appropriate registers.
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

    /// An ALU program that can be parsed from text input.
    pub struct Program {
        /// All the instructions in execution order.
        instructions: Vec<Instruction>,
    }
    impl FromStr for Program {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let instructions = Instruction::gather(s.lines().filter_map(|line| {
                let trimmed = line.trim();

                // Filter out blank and comment lines
                if trimmed.starts_with('#') || trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            }))?;

            Ok(Self { instructions })
        }
    }
    impl Program {
        /// Executes the ALU program given a number of inputs.
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

    /// Searches for a solution based on a digit iterator to determine order.
    ///
    /// Described further in the notes.
    pub fn find_solution(
        program: &Program,
        digit_iter: impl Iterator<Item = Number> + Clone,
    ) -> AocResult<Number> {
        /// This is an internal function of [`find_solution`].
        ///
        /// Determines a digit from the previous `z` and the `b` parameter.
        fn set_digit(z: Number, b: Number) -> Number {
            (z % 26) + b
        }

        /// This is an internal function of [`find_solution`].
        ///
        /// Determines the next `z` value based on the previous one.
        fn set_z(z: Number) -> Number {
            z / 26
        }

        /// This is an internal function of [`find_solution`].
        ///
        /// Tests whether a digit is valid (i.e. 1-9).
        fn valid_digit(d: Number) -> bool {
            (1..=9).contains(&d)
        }

        /// This is an internal function of [`find_solution`].
        ///
        /// Converts an array of digits into a number with the first digit being the
        /// least significant.
        fn digits_to_number(digits: &[Number]) -> Number {
            digits
                .iter()
                .rev()
                .enumerate()
                .map(|(n, d)| d * (10 as Number).pow(u32::try_from(n).unwrap()))
                .sum()
        }

        // Look for all potentially valid model numbers from the analysis (see the notes)
        for digs in (0..7).map(|_| digit_iter.clone()).multi_cartesian_product() {
            // Extract the digits
            let d1 = digs[0];
            let d2 = digs[1];
            let d3 = digs[2];
            let d5 = digs[3];
            let d6 = digs[4];
            let d9 = digs[5];
            let d11 = digs[6];

            // Digit 4
            let z3 = 676 * d1 + 26 * d2 + d3 + 2211;
            let d4 = set_digit(z3, -4);
            if !valid_digit(d4) {
                continue;
            }
            let z4 = set_z(z3);

            // Digit 7
            let z6 = 676 * z4 + 26 * d5 + d6 + 371;
            let d7 = set_digit(z6, -4);
            if !valid_digit(d7) {
                continue;
            }

            // Digit 8
            let z7 = set_z(z6);
            let d8 = set_digit(z7, -12);
            if !valid_digit(d8) {
                continue;
            }

            // Digit 10
            let z8 = set_z(z7);
            let z9 = 26 * z8 + d9 + 6;
            let d10 = set_digit(z9, -11);
            if !valid_digit(d10) {
                continue;
            }
            let z10 = set_z(z9);

            // Digit 12
            let z11 = 26 * z10 + d11;
            let d12 = set_digit(z11, -1);
            if !valid_digit(d12) {
                continue;
            }

            // Digit 13
            let z12 = set_z(z11);
            let d13 = set_digit(z12, 0);
            if !valid_digit(d13) {
                continue;
            }

            // Digit 14
            let z13 = set_z(z12);
            let d14 = set_digit(z13, -11);
            if !valid_digit(d14) {
                continue;
            }
            let z14 = set_z(z13);
            if z14 != 0 {
                continue;
            }

            // We should have a valid one!
            let digits = [d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12, d13, d14];

            // Verify that it is in fact valid
            assert_eq!(program.execute(&digits)?.value(Register::Z), 0);

            return Ok(digits_to_number(&digits));
        }

        Err(AocError::NoSolution)
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 24,
    name: "Arithmetic Logic Unit",
    preprocessor: Some(|input| Ok(Box::new(Program::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            find_solution(input.expect_data::<Program>()?, (1..=9).rev())
                .map(|n| Answer::Unsigned(n.try_into().unwrap()))
        },
        // Part one
        |input| {
            // Process
            find_solution(input.expect_data::<Program>()?, 1..=9)
                .map(|n| Answer::Unsigned(n.try_into().unwrap()))
        },
    ],
};
