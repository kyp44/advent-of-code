use std::{collections::HashMap, str::FromStr};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, one_of},
    combinator::map,
    multi::many_m_n,
    sequence::{preceded, tuple},
};

use crate::aoc::{AocError, ParseResult, Parseable, Solution};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![9967721333886],
    "mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X
mem[8] = 11
mem[7] = 101
mem[8] = 0",
        vec![165]
    }
}

const BITS: usize = 36;

#[derive(Debug)]
struct Mask {
    reset_mask: u64,
    set_mask: u64,
}

impl Parseable for Mask {
    fn parse(input: &str) -> ParseResult<Self> {
        fn vec_to_mask(vec: &Vec<char>, conv: fn(char) -> bool) -> u64 {
            let mut val = 0;
            for (bit, c) in vec.iter().rev().enumerate() {
                if conv(*c) {
                    val |= 1 << bit;
                }
            }
            val
        }

        map(many_m_n(BITS, BITS, one_of("X01")), |v| Mask {
            reset_mask: vec_to_mask(&v, |c| c == 'X'),
            set_mask: vec_to_mask(&v, |c| c == '1'),
        })(input)
    }
}

impl Default for Mask {
    fn default() -> Self {
        Mask {
            reset_mask: std::u64::MAX,
            set_mask: 0,
        }
    }
}

impl Mask {
    fn apply(&self, val: &u64) -> u64 {
        (self.reset_mask & *val) | self.set_mask
    }
}

#[derive(Debug)]
struct Memory {
    address: u64,
    value: u64,
}

impl Memory {
    fn new(address: u64, value: u64) -> Memory {
        fn check(val: u64) -> u64 {
            assert!(val < (1 << BITS), "Value of {} exceeds {} bits", val, BITS);
            val
        }
        Memory {
            address: check(address),
            value: check(value),
        }
    }
}

#[derive(Debug)]
enum Operation {
    SetMask(Mask),
    SetMemory(Memory),
}

impl Parseable for Operation {
    fn parse(input: &str) -> ParseResult<Self> {
        alt((
            map(preceded(tag("mask = "), Mask::parse), |m| {
                Operation::SetMask(m)
            }),
            map(
                tuple((tag("mem["), digit1, tag("] = "), digit1)),
                |(_, a, _, v): (&str, &str, &str, &str)| {
                    Operation::SetMemory(Memory::new(a.parse().unwrap(), v.parse().unwrap()))
                },
            ),
        ))(input.trim())
    }
}

struct Program {
    operations: Vec<Operation>,
}

impl FromStr for Program {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Program {
            operations: Operation::gather(s.lines())?,
        })
    }
}

impl Program {
    fn execute(&self) -> u64 {
        let mut mask = &Mask::default();
        let mut memory: HashMap<u64, u64> = HashMap::new();
        for op in self.operations.iter() {
            match op {
                Operation::SetMask(m) => mask = &m,
                Operation::SetMemory(m) => {
                    memory.insert(m.address, mask.apply(&m.value));
                }
            }
        }
        memory.iter().map(|(_, v)| v).sum()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 14,
    name: "Docking Data",
    solver: |input| {
        // Generation
        let program: Program = input.parse()?;

        // Process
        let answers = vec![program.execute()];

        Ok(answers)
    },
};
