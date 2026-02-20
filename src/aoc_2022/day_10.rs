use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use Answer::{Signed, Unsigned};
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "noop
addx 3
addx -5";
            answers = &[Some(Signed(0)), None];
        }
        example {
            input = "addx 15
addx -11
addx 6
addx -3
addx 5
addx -1
addx -8
addx 13
addx 4
noop
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx -35
addx 1
addx 24
addx -19
addx 1
addx 16
addx -11
noop
noop
addx 21
addx -15
noop
noop
addx -3
addx 9
addx 1
addx -3
addx 8
addx 1
addx 5
noop
noop
noop
noop
noop
addx -36
noop
addx 1
addx 7
noop
noop
noop
addx 2
addx 6
noop
noop
noop
noop
noop
addx 1
noop
noop
addx 7
addx 1
noop
addx -13
addx 13
addx 7
noop
addx 1
addx -33
noop
noop
noop
addx 2
noop
noop
noop
addx 8
noop
addx -1
addx 2
addx 1
noop
addx 17
addx -9
addx 1
addx 1
addx -3
addx 11
noop
noop
addx 1
noop
addx 1
noop
noop
addx -13
addx -19
addx 1
addx 3
addx 26
addx -30
addx 12
addx -1
addx 3
addx 1
noop
noop
noop
addx -9
addx 18
addx 1
addx 2
noop
noop
addx 9
noop
noop
noop
addx -1
addx 2
addx -37
addx 1
addx 3
noop
addx 15
addx -21
addx 22
addx -6
addx 1
noop
addx 2
addx 1
noop
addx -10
noop
noop
addx 20
addx 1
addx 2
addx 2
addx -6
addx -11
noop
noop
noop";
            answers = answers![Signed(13140), Unsigned(124)];
        }
        actual_answers = answers![Signed(17940), Unsigned(92)];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::slice::Iter;

    use super::*;
    use aoc::grid::StdBool;
    use nom::{
        branch::alt, bytes::complete::tag, character::complete::space1, combinator::map,
        sequence::separated_pair,
    };

    /// A single CPU instruction.
    #[derive(Debug)]
    enum Instruction {
        /// No operation, that is, do nothing for a single cycle.
        Noop,
        /// Add something to the x register, takes two cycles before the number
        /// is actually added.
        Add(i64),
    }
    impl Parsable for Instruction {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            alt((
                map(tag("noop"), |_| Self::Noop),
                map(
                    separated_pair(tag("addx"), space1, nom::character::complete::i64),
                    |(_, n)| Self::Add(n),
                ),
            ))
            .parse(input)
        }
    }

    /// A program for the CPU to execute.
    #[derive(Debug)]
    pub struct Program {
        /// The list of instructions to execute.
        instructions: Vec<Instruction>,
    }
    impl FromStr for Program {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                instructions: Instruction::gather(s.lines())?,
            })
        }
    }
    impl Program {
        /// Returns an [`Executor`] iterator to execute the program.
        pub fn execute(&self) -> Executor<'_> {
            Executor {
                instructions: self.instructions.iter(),
                cpu_state: Default::default(),
                current_add: None,
            }
        }
    }

    /// The state of an add x instruction.
    struct AddInstruction {
        /// The number to add when complete.
        to_add: i64,
        /// The number of cycles left before the instruction is complete and the
        /// number is added.
        cycles_left: usize,
    }
    impl AddInstruction {
        /// Initializes a new instruction state `to_add` a particular number.
        pub fn new(to_add: i64) -> Self {
            Self {
                to_add,
                // The add instructions always takes two cycles to complete.
                cycles_left: 2,
            }
        }
    }

    /// A state of the CPU.
    #[derive(Clone)]
    pub struct CpuState {
        /// The cycle that just completed.
        cycle: usize,
        /// The x register during the cycle that just completed.
        register_x: i64,
    }
    impl Default for CpuState {
        fn default() -> Self {
            Self {
                cycle: 0,
                register_x: 1,
            }
        }
    }
    impl CpuState {
        /// Increments the cycle.
        fn tick(&mut self) {
            self.cycle += 1;
        }

        /// Adds a value directly to the x register.
        fn add(&mut self, n: i64) {
            self.register_x += n;
        }

        /// Calculates the signal strength for this CPU state, that is the cycle
        /// number times the x register.
        pub fn signal_strength(&self) -> i64 {
            i64::try_from(self.cycle).unwrap() * self.register_x
        }
    }

    /// An [`Iterator`] over the CPU states after each cycle as a program is
    /// executed.
    ///
    /// This should only be created using [`Program::execute`].
    /// Note that the first state will be after the first cycle, not the initial
    /// state.
    pub struct Executor<'a> {
        /// The list of instructions to execute.
        instructions: Iter<'a, Instruction>,
        /// The current CPU state.
        cpu_state: CpuState,
        /// The current add instruction state if we are currently executing an
        /// add instruction.
        current_add: Option<AddInstruction>,
    }
    impl Iterator for Executor<'_> {
        type Item = CpuState;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(ai) = self.current_add.as_mut() {
                ai.cycles_left -= 1;

                if ai.cycles_left == 0 {
                    // Apply add, and we still want to fetch the next instruction below here
                    self.cpu_state.add(ai.to_add);
                    self.current_add = None;
                } else {
                    self.cpu_state.tick();
                    return Some(self.cpu_state.clone());
                }
            }

            // Fetch the next instruction
            self.instructions.next().map(|inst| {
                match inst {
                    Instruction::Noop => {}
                    Instruction::Add(n) => self.current_add = Some(AddInstruction::new(*n)),
                }
                self.cpu_state.tick();
                self.cpu_state.clone()
            })
        }
    }

    /// Renders the CRT pixels generated from executing a given [`Program`].
    pub fn render_crt(program: &Program) -> Grid<StdBool> {
        let size = GridSize::new(40, 6);
        let mut pixels = Grid::default(size);

        for (cpu, point) in program.execute().zip(pixels.all_points()) {
            let sprite = (cpu.register_x - 1)..=(cpu.register_x + 1);
            if sprite.contains(&i64::try_from(point.x).unwrap()) {
                pixels.set(&point, true.into());
            }
        }

        pixels
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 10,
    name: "Cathode-Ray Tube",
    preprocessor: Some(|input| Ok(Box::new(Program::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(Answer::Signed(
                input
                    .expect_data::<Program>()?
                    .execute()
                    .skip(19)
                    .step_by(40)
                    .map(|cpu| cpu.signal_strength())
                    .sum(),
            ))
        },
        // Part two
        |input| {
            let pixels = render_crt(input.expect_data::<Program>()?);

            // This requires looking at letters in the folded image,
            // which cannot really be done in automated way easily.
            println!("Part two image:\n");
            println!("{pixels:?}");
            println!("Part two actual answer: ZCBAJFJZ\n");

            // Process
            Ok(Answer::Unsigned(pixels.all_values().filter_count(|p| ***p)))
        },
    ],
};
