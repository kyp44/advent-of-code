//! Collection of items that abstract over the idea of a program that can be
//! executed as a series of instructions.
//!
//! This need not be simply be a toy computer program, but applies to anything
//! where instructions are followed to modify the state of something.
use derive_new::new;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    iter::FusedIterator,
};

use crate::{parse::Parsable, prelude::AocResult};

/// A set of simple registers that can be modeled as a map of keys to
/// the stored register values.
pub trait SimpleRegisters {
    /// The register key type.
    type Key: Eq + Hash;
    /// The register value type.
    type Value;

    /// Returns the underlying map.
    fn map(&self) -> &HashMap<Self::Key, Self::Value>;

    /// Returns a mutable reference to the underlying map.
    fn map_mut(&mut self) -> &mut HashMap<Self::Key, Self::Value>;

    /// Returns the register value for a given `key`.
    fn get(&self, key: &Self::Key) -> &Self::Value {
        self.map().get(key).unwrap()
    }

    /// Returns a mutable reference to the register value for a given `key`.
    fn get_mut(&mut self, key: &Self::Key) -> &mut Self::Value {
        self.map_mut().get_mut(key).unwrap()
    }

    /// Sets a register `value` for the given `key`.
    fn set(&mut self, key: Self::Key, val: Self::Value) -> Self::Value {
        self.map_mut().insert(key, val).unwrap()
    }

    /// Modifies a register value for a given `key` in place.
    fn modify(&mut self, key: Self::Key, f: impl FnOnce(&Self::Value) -> Self::Value) {
        self.map_mut().entry(key).and_modify(|v| *v = f(v));
    }
}

/// A jump in the program instructions.
pub enum Jump {
    /// An absolute jump with the index of the instruction to which to jump.
    Absolute(usize),
    /// The index of the instruction to which to jump relative to the current
    /// instruction, with zero being the current instruction.
    ///
    /// A negative number jumps to a preceding instruction, while a positive
    /// number jumps to a succeeding instruction.
    Relative(isize),
}

/// The structure returned from the execution of a single [`Instruction`].
///
/// This implements [`Default`], which returns the default `yielded_item` with
/// no jump.
#[derive(new, Default)]
pub struct Executed<Y> {
    /// The item yielded by the instruction execution.
    pub yielded_item: Y,
    /// How the program should jump, or `None` if execution should simply move
    /// to the next [`Instruction`].
    pub jump: Option<Jump>,
}
impl<Y> Executed<Y> {
    /// Creates a new structure with a `yielded` item and no jump.
    pub fn no_jump(yielded: Y) -> Self {
        Self::new(yielded, None)
    }
}
impl<Y: Default> Executed<Y> {
    /// Creates a new structure with the default `yielded_item` and a possible
    /// [`Jump`].
    pub fn only_jump(jump: Option<Jump>) -> Self {
        Self::new(Y::default(), jump)
    }
}

/// An abstract instruction that can be executed.
///
/// # Examples
///
/// Refer to the [2015 day 23
/// problem](../../advent_of_code/aoc_2015/day_23/solution/index.html) or the
/// [2020 day 12
/// problem](../../advent_of_code/aoc_2020/day_12/solution/index.html) for
/// examples of instruction sets.
pub trait Instruction {
    /// The type that can be mutated during execution.
    type Registers;
    /// An item yielded by the execution.
    type YieldItem;
    /// The error type if execution fails.
    type Err;

    /// Executes this instruction, operating on the `registers` and returning a
    /// yielded item and a possible [`Jump`].
    fn execute(
        &self,
        registers: &mut Self::Registers,
    ) -> Result<Executed<Self::YieldItem>, Self::Err>;
}

/// Possible ways for a program to end.
#[derive(Clone, Copy, Debug)]
pub enum ProgramEndStatus {
    /// Jumped outside the bounds of the program instructions.
    JumpedOut,
    /// Terminated normally, after executing the final instruction.
    Terminated,
    /// Detected an infinite loop.
    Infinite,
}

/// A program state.
#[derive(new, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProgramState<R> {
    /// The current program counter, that is the instruction index.
    pub program_counter: usize,
    /// The current state of the registers.
    pub registers: R,
}

/// The results of a program after complete execution.
#[derive(new, Debug)]
pub struct ProgramEnd<R, Y> {
    /// The final state.
    pub last_state: ProgramState<R>,
    /// The item yielded by the final instruction that was executed.
    pub last_yielded: Option<Y>,
}
impl<R, Y> ProgramEnd<R, Y> {
    /// Keeps only the final state of the registers.
    pub fn into_registers(self) -> R {
        self.last_state.registers
    }

    /// Accesses the final state of the registers.
    pub fn registers(&self) -> &R {
        &self.last_state.registers
    }
}

/// The results of a program after complete execution, which also tracks how the
/// program ended.
#[derive(new, Debug)]
pub struct MonitoredProgramEnd<R, Y> {
    /// The normal program results.
    pub program_end: ProgramEnd<R, Y>,
    /// The manner in which the program ended.
    pub end_status: ProgramEndStatus,
}
impl<R, Y> MonitoredProgramEnd<R, Y> {
    /// Accesses the final state of the registers.
    pub fn registers(&self) -> &R {
        &self.program_end.last_state.registers
    }
}

/// A program, which is just a sequence of instructions of type `I`.
///
/// Can be parsed from text input if `I` can be.
///
/// # Examples
///
/// Refer to the [2015 day 23
/// problem](../../advent_of_code/aoc_2015/day_23/solution/index.html) or the
/// [2020 day 12
/// problem](../../advent_of_code/aoc_2020/day_12/solution/index.html) for
/// examples of programs.
#[derive(Clone, Debug)]
pub struct Program<I> {
    /// The list of instructions.
    instructions: Vec<I>,
}
impl<I> Program<I> {
    /// Creates the program directly from the list of instructions.
    pub fn new(instructions: Vec<I>) -> Self {
        Self { instructions }
    }

    /// Returns the list of instructions as a slice.
    pub fn instructions(&self) -> &[I] {
        &self.instructions
    }
}
impl<I: Parsable> Program<I> {
    /// Parses the list of instructions from text input, assuming that each
    /// instructions is on its own line in the `input`.
    pub fn parse<'a>(input: &'a str) -> AocResult<Self>
    where
        I::Parsed<'a>: Into<I>,
    {
        Ok(Self::new(
            I::gather(input.lines())?
                .into_iter()
                .map(|inst| inst.into())
                .collect(),
        ))
    }
}
impl<I: Instruction> Program<I> {
    /// Returns an executor for this program given an initial state of the
    /// registers.
    pub fn executor(&self, initial_registers: I::Registers) -> ProgramExecutor<'_, I> {
        ProgramExecutor {
            program: self,
            registers: initial_registers,
            program_counter: 0,
            jumped: false,
        }
    }

    /// Executes the program to completion.
    ///
    /// This fails as soon as any of the instruction executions fail.
    pub fn execute(
        &self,
        initial_registers: I::Registers,
    ) -> Result<ProgramEnd<I::Registers, I::YieldItem>, I::Err> {
        let mut executor = self.executor(initial_registers);
        let mut last_yielded = None;

        loop {
            let last_pc = executor.program_counter;

            match executor.next() {
                Some(y) => last_yielded = Some(y?),
                None => {
                    break Ok(ProgramEnd::new(
                        ProgramState::new(last_pc, executor.registers),
                        last_yielded,
                    ));
                }
            }
        }
    }
}
impl<I: Instruction> Program<I>
where
    I::Registers: Clone + std::fmt::Debug + Eq + Hash,
{
    /// Executes a program to completion, monitoring the way that the program
    /// terminates.
    ///
    /// An infinite loop is detected if the program is about to execute the
    /// same instruction in the program while the registers are identical.
    ///
    /// This fails as soon as any of the instruction executions fail.
    pub fn execute_monitored(
        &self,
        initial_registers: I::Registers,
    ) -> Result<MonitoredProgramEnd<I::Registers, I::YieldItem>, I::Err> {
        let mut visited_states = HashSet::new();
        let mut executor = self.executor(initial_registers);
        let mut last_yielded = None;

        loop {
            let current_state =
                ProgramState::new(executor.program_counter, executor.registers.clone());

            // Add the current state
            if !visited_states.insert(current_state.clone()) {
                // In an infinite loop
                break Ok(MonitoredProgramEnd::new(
                    ProgramEnd::new(current_state, last_yielded),
                    ProgramEndStatus::Infinite,
                ));
            }

            // Execute the next instruction
            if executor
                .next()
                .transpose()?
                .map(|yi| last_yielded = Some(yi))
                .is_none()
            {
                // The program is complete, so did we jump out or finish normally?
                break Ok(if executor.jumped {
                    MonitoredProgramEnd::new(
                        ProgramEnd::new(current_state, last_yielded),
                        ProgramEndStatus::JumpedOut,
                    )
                } else {
                    MonitoredProgramEnd::new(
                        ProgramEnd::new(current_state, last_yielded),
                        ProgramEndStatus::Terminated,
                    )
                });
            }
        }
    }
}

/// An execution [`Iterator`] over the program instructions.
pub struct ProgramExecutor<'a, I: Instruction> {
    /// The program being executed.
    program: &'a Program<I>,
    /// The state of the registers after the most recent instruction was
    /// executed.
    pub registers: I::Registers,
    /// Index of the _next_ instruction to be executed.
    pub program_counter: usize,
    /// Whether the most recent instruction caused a jump.
    pub jumped: bool,
}
impl<'a, I: Instruction> ProgramExecutor<'a, I> {
    /// Returns the next instruction that needs to be executed, if there is one.
    pub fn next_instruction(&self) -> Option<&'a I> {
        (self.program_counter < self.program.instructions.len())
            .then(|| &self.program.instructions[self.program_counter])
    }
}
impl<'a, I: Instruction> Iterator for ProgramExecutor<'a, I> {
    type Item = Result<I::YieldItem, I::Err>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(
            self.next_instruction()
                .map(|inst| inst.execute(&mut self.registers))?
                .map(|exec| {
                    self.program_counter = match exec.jump {
                        Some(jump) => {
                            self.jumped = true;
                            match jump {
                                Jump::Absolute(pc) => pc,
                                Jump::Relative(delta) => {
                                    let pc = isize::try_from(self.program_counter).unwrap() + delta;

                                    if pc < 0 { 0 } else { pc.try_into().unwrap() }
                                }
                            }
                        }
                        None => {
                            self.jumped = false;
                            self.program_counter + 1
                        }
                    };

                    exec.yielded_item
                }),
        )
    }
}
impl<I: Instruction> FusedIterator for ProgramExecutor<'_, I> {}
