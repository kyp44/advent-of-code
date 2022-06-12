use crate::aoc::prelude::*;
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::one_of,
    combinator::map,
    multi::many_m_n,
    sequence::{preceded, tuple},
};
use std::{collections::HashMap, str::FromStr};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(9967721333886), Unsigned(4355897790573)],
    "mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X
mem[8] = 11
mem[7] = 101
mem[8] = 0",
        vec![Some(Unsigned(165)), None],
    "mask = 000000000000000000000000000000X1001X
        mem[42] = 100
        mask = 00000000000000000000000000000000X0XX
        mem[26] = 1",
    vec![None, Some(Unsigned(208))]
    }
}

const BITS: usize = 36;

#[derive(Debug)]
enum MaskBit {
    X,
    Zero,
    One,
}
impl From<char> for MaskBit {
    fn from(c: char) -> Self {
        match c {
            '0' => MaskBit::Zero,
            '1' => MaskBit::One,
            'X' => MaskBit::X,
            _ => panic!("Unkown mask bit type {}", c),
        }
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
    SetMask(Vec<MaskBit>),
    SetMemory(Memory),
}

impl Parseable<'_> for Operation {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        use nom::character::complete::u64 as cu64;
        alt((
            map(
                preceded(tag("mask = "), many_m_n(BITS, BITS, one_of("X01"))),
                |v: Vec<char>| Operation::SetMask(v.into_iter().rev().map(MaskBit::from).collect()),
            ),
            map(
                tuple((tag("mem["), cu64, tag("] = "), cu64)),
                |(_, a, _, v)| Operation::SetMemory(Memory::new(a, v)),
            ),
        ))(input.trim())
    }
}

trait Mask: for<'a> From<&'a [MaskBit]> + Default {
    fn apply(&self, memory: &Memory) -> Vec<Memory>;

    fn vec_to_mask(vec: &[MaskBit], mut conv: impl FnMut(&MaskBit) -> bool) -> u64 {
        let mut val = 0;
        for (bit, mb) in vec.iter().enumerate() {
            if conv(mb) {
                val |= 1 << bit;
            }
        }
        val
    }
}

// Mask for decoder chip v1 (Part a)
struct MaskV1 {
    reset_mask: u64,
    set_mask: u64,
}
impl From<&[MaskBit]> for MaskV1 {
    fn from(v: &[MaskBit]) -> Self {
        assert_eq!(v.len(), BITS);
        MaskV1 {
            reset_mask: Self::vec_to_mask(v, |mb| matches!(mb, MaskBit::X)),
            set_mask: Self::vec_to_mask(v, |mb| matches!(mb, MaskBit::One)),
        }
    }
}
impl Default for MaskV1 {
    fn default() -> Self {
        MaskV1 {
            reset_mask: std::u64::MAX,
            set_mask: 0,
        }
    }
}
impl Mask for MaskV1 {
    fn apply(&self, memory: &Memory) -> Vec<Memory> {
        vec![Memory::new(
            memory.address,
            (self.reset_mask & memory.value) | self.set_mask,
        )]
    }
}

// Mask for decoder chip v2 (Part b)
struct MaskV2 {
    reset_mask: u64,
    set_masks: Vec<u64>,
}
impl From<&[MaskBit]> for MaskV2 {
    fn from(v: &[MaskBit]) -> Self {
        assert_eq!(v.len(), BITS);
        MaskV2 {
            reset_mask: Self::vec_to_mask(v, |mb| matches!(mb, MaskBit::Zero)),
            set_masks: {
                let num_floating: usize = v.iter().filter_count(|mb| matches!(mb, MaskBit::X));
                (0..num_floating)
                    .map(|_| &[false, true])
                    .multi_cartesian_product()
                    .map(|tf_vec| {
                        let mut tf = tf_vec.into_iter();
                        Self::vec_to_mask(v, move |mb| match mb {
                            MaskBit::Zero => false,
                            MaskBit::One => true,
                            MaskBit::X => *tf.next().unwrap(),
                        })
                    })
                    .collect()
            },
        }
    }
}
impl Default for MaskV2 {
    fn default() -> Self {
        MaskV2 {
            reset_mask: std::u64::MAX,
            set_masks: vec![0],
        }
    }
}
impl Mask for MaskV2 {
    fn apply(&self, memory: &Memory) -> Vec<Memory> {
        self.set_masks
            .iter()
            .map(|sm| Memory {
                address: (memory.address & self.reset_mask) | sm,
                value: memory.value,
            })
            .collect()
    }
}

#[derive(Debug)]
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
    fn execute<M: Mask>(&self) -> u64 {
        let mut mask = M::default();
        let mut memory: HashMap<u64, u64> = HashMap::new();
        for op in self.operations.iter() {
            match op {
                Operation::SetMask(mv) => mask = mv[..].into(),
                Operation::SetMemory(m) => {
                    for mem in mask.apply(m) {
                        memory.insert(mem.address, mem.value);
                    }
                }
            }
        }
        memory.iter().map(|(_, v)| v).sum()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 14,
    name: "Docking Data",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let program: Program = input.expect_input()?.parse()?;

            // Process
            Ok(program.execute::<MaskV1>().into())
        },
        // Part b)
        |input| {
            // Generation
            let program: Program = input.expect_input()?.parse()?;

            // Process
            Ok(program.execute::<MaskV2>().into())
        },
    ],
};
