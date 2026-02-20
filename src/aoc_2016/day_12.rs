use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "";
            answers = unsigned![123];
        }
        actual_answers = unsigned![123];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;

    type Jump = usize;

    trait Instruction: Parsable {
        type Registers;

        fn execute(&self, registers: &mut Self::Registers) -> Option<Jump>;
    }

    struct Processor<R, I> {
        registers: R,
        instructions: Vec<I>,
    }
    impl<R, I: Instruction> Processor<R, I> {
        pub fn new(initial_registers: R, instructions: Vec<I>) -> Self {
            Self {
                registers: initial_registers,
                instructions,
            }
        }

        pub fn from_str<'a>(input: &'a str, initial_registers: R) -> AocResult<Self>
        where
            I::Parsed<'a>: Into<I>,
        {
            Ok(Self {
                registers: initial_registers,
                instructions: I::gather(input.lines())?
                    .into_iter()
                    .map(|inst| inst.into())
                    .collect(),
            })
        }

        pub fn execute
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 12,
    name: "Leonardo's Monorail",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation

            // Process
            Ok(0u64.into())
        },
    ],
};
