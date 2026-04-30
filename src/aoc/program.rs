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

/// A program counter along with the set of instructions to which the counter
/// applies.
///
/// The counter may be valid, pointing to an instruction, or can be invalid.
/// The counter will become invalid if an instruction jumps outside the
/// program, or if it is incremented past the last instruction.
///
/// Once invalid, the only way for it to become valid again is to call
/// [`jump_absolute`](ProgramCounter::jump_absolute) with a valid `index`.
pub struct ProgramCounter<I: Instruction> {
    /// The list of program instructions, which could be mutated.
    instructions: Vec<I>,
    /// The index of the current instruction in `instructions`, or `None` if the
    /// counter has been invalidated.
    index: Option<usize>,
    /// The last instruction jumped to another instruction.
    jumped: bool,
}
impl<I: Instruction> From<Vec<I>> for ProgramCounter<I> {
    fn from(value: Vec<I>) -> Self {
        Self {
            instructions: value,
            index: Some(0),
            jumped: false,
        }
    }
}
impl<I: Instruction> ProgramCounter<I> {
    /// Returns a slice of the instruction list.
    pub fn instructions(&self) -> &[I] {
        &self.instructions
    }

    /// Sets the counter to a specific `index`, invaliding it if `index` is out
    /// of range.
    fn set(&mut self, index: usize) {
        self.index = (index < self.instructions.len()).then_some(index);
    }

    /// Returns the current index if valid and `None` otherwise.
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    /// Returns the current instruction, or `None` if the counter is
    /// invalid.
    pub fn current_instruction(&self) -> Option<&I> {
        self.index.map(|idx| &self.instructions[idx])
    }

    /// Returns whether the last instruction caused teh counter to jump.
    pub fn jumped(&self) -> bool {
        self.jumped
    }

    /// Increments the counter, invalidating it if the current instruction is
    /// the last one.
    pub fn increment(&mut self) {
        if let Some(idx) = self.index {
            self.set(idx + 1);
        }
        self.jumped = false;
    }

    /// Returns the index of some instruction relative to the current
    /// instruction.
    ///
    /// This will be `None` if the index is outside the valid range.
    pub fn relative_index(&self, offset: isize) -> Option<usize> {
        let curr_index = isize::try_from(self.index?).ok()?;
        let index = usize::try_from(curr_index + offset).ok()?;

        (index < self.instructions.len()).then_some(index)
    }

    /// Jumps to a specific instruction, invalidating the counter if it is
    /// outside the valid range.
    ///
    /// This can revalidate the counter if it is invalid and a valid `index` is
    /// passed.
    pub fn jump_absolute(&mut self, index: usize) {
        self.set(index);
        self.jumped = true;
    }

    /// Jumps to an instruction relative to the current instruction,
    /// invalidating the counter if this is out of the valid range.
    pub fn jump_relative(&mut self, offset: isize) {
        self.index = self.relative_index(offset);
        self.jumped = true;
    }

    /// Jumps to a relative index or simply increments the program counter if
    /// `jump` is `None`.
    ///
    /// This can of course invalidate the counter if the jump index is out of
    /// range.
    pub fn jump_relative_or_increment(&mut self, jump: Option<isize>) {
        match jump {
            Some(o) => self.jump_relative(o),
            None => self.increment(),
        }
    }

    /// Modifies the instruction using a closure.
    ///
    /// This does not change the current counter `index` even if the underlying
    /// instructions change. This can invalidate the counter if enough
    /// instructions are removed such that the current index becomes out of
    /// range.
    pub fn change_instructions(&mut self, f: impl FnOnce(&mut Vec<I>)) {
        f(&mut self.instructions);

        // Re-asses the current index in case the size of the instruction list changed
        if let Some(idx) = self.index {
            self.set(idx);
        }
    }

    /// Allows execution of a different instruction type `O` by providing a
    /// dummy program counter to a closure pass to
    /// [`O::execute`](Instruction::execute).
    ///
    /// This allows the instruction to modify the dummy program counter, which
    /// is then applied back to this counter.
    pub fn with_dummy<O: Instruction + Default>(
        &mut self,
        f: impl FnOnce(&mut ProgramCounter<O>) -> Result<O::YieldItem, O::Err>,
    ) -> Result<O::YieldItem, O::Err> {
        let mut dummy_pc = ProgramCounter::from(vec![O::default(); self.instructions().len()]);
        dummy_pc.index = self.index;

        let res = f(&mut dummy_pc);

        self.index = dummy_pc.index;
        res
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
/// examples of basic instruction sets.
/// The [2016 day 23
/// problem](../../advent_of_code/aoc_2016/day_23/solution/index.html) is an
/// example of an instruction set that requires mutating the program.
pub trait Instruction: Sized + Clone {
    /// The type that can be mutated during execution.
    type Registers;
    /// An item yielded by the execution.
    type YieldItem;
    /// The error type if execution fails.
    type Err;

    /// Executes this instruction, operating on the `registers` and returning a
    /// yielded item.
    ///
    /// Most instructions will require a `program_counter`, and executing
    /// a standard [`Program`] will always pass a `program_counter`. Custom
    /// programs that execute their own instructions may not require one. For an
    /// example of this, see the [2016 day 10
    /// problem](../../advent_of_code/aoc_2016/day_10/solution/index.html).
    ///
    /// If a `program_counter` is passed, the execution is expected to modify it
    /// to at least [`increment`](ProgramCounter::increment) it to the next
    /// instruction. The instructions themselves may also be mutated by calling
    /// [`ProgramCounter::change_instructions`].
    fn execute(
        &self,
        program_counter: Option<&mut ProgramCounter<Self>>,
        registers: &mut Self::Registers,
    ) -> Result<Self::YieldItem, Self::Err>;
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

/// The results of a program after complete execution.
#[derive(new, Debug)]
pub struct ProgramEnd<R, Y> {
    /// The final state.
    pub registers: R,
    /// The item yielded by the final instruction that was executed.
    pub last_yielded: Option<Y>,
}
impl<R, Y> ProgramEnd<R, Y> {
    /// Keeps only the final state of the registers.
    pub fn into_registers(self) -> R {
        self.registers
    }

    /// Accesses the final state of the registers.
    pub fn registers(&self) -> &R {
        &self.registers
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
        &self.program_end.registers
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
/// examples.
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
    pub fn executor(&self, initial_registers: I::Registers) -> ProgramExecutor<I> {
        ProgramExecutor {
            program_counter: ProgramCounter::from(self.instructions.clone()),
            registers: initial_registers,
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
            match executor.next() {
                Some(y) => {
                    last_yielded = Some(y?);
                }
                None => {
                    break Ok(ProgramEnd::new(executor.registers, last_yielded));
                }
            }
        }
    }
}
impl<I: Instruction> Program<I>
where
    I::Registers: Clone + std::fmt::Debug + Eq + Hash,
{
    /// Returns a monitored executor for this program given an initial state of
    /// the registers.
    pub fn monitored_executor(
        &self,
        initial_registers: I::Registers,
    ) -> MonitoredProgramExecutor<I> {
        MonitoredProgramExecutor {
            executor: self.executor(initial_registers),
            visited_states: HashSet::new(),
        }
    }

    /// Executes a program to completion, monitoring the way that the program
    /// terminates.
    ///
    /// An infinite loop is detected if the program is about to execute the
    /// same instruction in the program while the registers are identical.
    ///
    /// This fails as soon as any of the instruction executions fail.
    pub fn monitored_execute(
        &self,
        initial_registers: I::Registers,
    ) -> Result<MonitoredProgramEnd<I::Registers, I::YieldItem>, I::Err> {
        let mut mon_executor = self.monitored_executor(initial_registers);
        let mut last_yielded = None;

        loop {
            // Execute the next instruction
            match mon_executor.next().transpose()? {
                Some(inst_end) => {
                    last_yielded = Some(inst_end.yielded_item);

                    // Are we in an infinite loop?
                    if inst_end.repeated_state {
                        break Ok(MonitoredProgramEnd::new(
                            ProgramEnd::new(mon_executor.executor.registers, last_yielded),
                            ProgramEndStatus::Infinite,
                        ));
                    }
                }
                None => {
                    // The program is complete, so did we jump out or finish normally?
                    let program_end =
                        ProgramEnd::new(mon_executor.executor.registers, last_yielded);

                    break Ok(if mon_executor.executor.program_counter.jumped() {
                        MonitoredProgramEnd::new(program_end, ProgramEndStatus::JumpedOut)
                    } else {
                        MonitoredProgramEnd::new(program_end, ProgramEndStatus::Terminated)
                    });
                }
            }
        }
    }
}

/// An execution [`Iterator`] over the program instructions.
pub struct ProgramExecutor<I: Instruction> {
    /// The program counter.
    program_counter: ProgramCounter<I>,
    /// The registers on which the instructions act.
    registers: I::Registers,
}
impl<I: Instruction> Iterator for ProgramExecutor<I> {
    type Item = Result<I::YieldItem, I::Err>;

    fn next(&mut self) -> Option<Self::Item> {
        let inst = self.program_counter.current_instruction().cloned()?;

        Some(inst.execute(Some(&mut self.program_counter), &mut self.registers))
    }
}
impl<I: Instruction> FusedIterator for ProgramExecutor<I> {}

/// Returned at each step when executing a [`MonitoredProgramExecutor`].
pub struct MonitoredInstructionEnd<Y> {
    /// The item yielded from the instruction just executed.
    pub yielded_item: Y,
    /// Whether or not the program is in a state that it has been in before
    /// after the last instruction executed.
    ///
    /// This means that the program will never terminate and will loop
    /// infinitely.
    pub repeated_state: bool,
}

/// An execution [`Iterator`] over the program instructions that also monitors
/// for repeated states.
///
/// This involves some memory and execution time overhead so that
/// [`ProgramExecutor`] should be used instead if possible.
pub struct MonitoredProgramExecutor<I: Instruction> {
    /// The underlying standard executor.
    executor: ProgramExecutor<I>,
    /// The set of program states that have already been visited.
    ///
    /// In the tuple, the first element is the program counter, while the second
    /// is obviously the registers.
    visited_states: HashSet<(Option<usize>, I::Registers)>,
}
impl<I: Instruction> Iterator for MonitoredProgramExecutor<I>
where
    I::Registers: Clone + std::fmt::Debug + Eq + Hash,
{
    type Item = Result<MonitoredInstructionEnd<I::YieldItem>, I::Err>;

    fn next(&mut self) -> Option<Self::Item> {
        let exec_result = self.executor.next()?;

        // Add the current state and check whether we have already been here
        let repeated_state = !self.visited_states.insert((
            self.executor.program_counter.index(),
            self.executor.registers.clone(),
        ));

        Some(exec_result.map(|yielded_item| MonitoredInstructionEnd {
            yielded_item,
            repeated_state,
        }))
    }
}
