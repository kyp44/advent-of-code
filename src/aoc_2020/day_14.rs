use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
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

/// Contains solution implementation items.
mod solution {
    use super::*;
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

    /// Bit depth of the system.
    const BITS: usize = 36;

    /// Bit mask bit value, which can be parsed from a text character.
    #[derive(Debug)]
    pub enum MaskBit {
        /// Leave value bit unchanged (V1) or address bit floating, which is all possible values (V2).
        X,
        /// Reset value bit (V1) or leave address bit unchanged (V2).
        Zero,
        /// Set value bit (V1) or set address bit (V2).
        One,
    }
    impl From<char> for MaskBit {
        fn from(c: char) -> Self {
            match c {
                '0' => MaskBit::Zero,
                '1' => MaskBit::One,
                'X' => MaskBit::X,
                _ => panic!("Unkown mask bit type {c}"),
            }
        }
    }

    /// Information needed to assign value to a memory location.
    #[derive(Debug)]
    pub struct Assignment {
        /// Memory address to set.
        address: u64,
        /// Value to set.
        value: u64,
    }
    impl Assignment {
        /// Creates an assignment, validating the values.
        ///
        /// Panics if any values exceed the system bit depth.
        fn new(address: u64, value: u64) -> Assignment {
            /// Verifies that a number does not exceed the system bit depth.
            ///
            /// Sub-function of [`Assignment::new`] that panics if this is the
            /// case or simply returns the same number otherwise.
            fn check(val: u64) -> u64 {
                assert!(val < (1 << BITS), "Value of {val} exceeds {BITS} bits");
                val
            }
            Assignment {
                address: check(address),
                value: check(value),
            }
        }
    }

    /// A memory operation, which can be parsed from text input.
    #[derive(Debug)]
    enum Operation {
        /// Set the bit mask.
        SetMask(Vec<MaskBit>),
        /// Sets a memory location.
        SetMemory(Assignment),
    }
    impl Parseable<'_> for Operation {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            use nom::character::complete::u64 as cu64;
            alt((
                map(
                    preceded(tag("mask = "), many_m_n(BITS, BITS, one_of("X01"))),
                    |v: Vec<char>| {
                        Operation::SetMask(v.into_iter().rev().map(MaskBit::from).collect())
                    },
                ),
                map(
                    tuple((tag("mem["), cu64, tag("] = "), cu64)),
                    |(_, a, _, v)| Operation::SetMemory(Assignment::new(a, v)),
                ),
            ))(input.trim())
        }
    }

    /// Mask with behavior specific to a particular decoder chip version (problem part).
    pub trait Mask: for<'a> From<&'a [MaskBit]> + Default {
        /// Apply the mask to an assignment request according the rules for the version.
        ///
        /// Returns a list of hard assignments to make.
        fn apply_mask(&self, memory: &Assignment) -> Vec<Assignment>;

        /// Converts a list of bit mask values to a real bit mask using a bit conversion function.
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

    /// Mask for decoder chip version 1 (part one).
    pub struct MaskV1 {
        /// Real bit mask used to reset all value bits that need set to 0 (via bitwise AND).
        reset_mask: u64,
        /// Real bit mask used to set all value bits that need set to 1 (via bitwise OR).
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
        fn apply_mask(&self, memory: &Assignment) -> Vec<Assignment> {
            vec![Assignment::new(
                memory.address,
                (self.reset_mask & memory.value) | self.set_mask,
            )]
        }
    }

    /// Mask for decoder chip version 2 (part two).
    pub struct MaskV2 {
        /// Real bit mask used to reset all address floating bits (via bitwise AND).
        reset_mask: u64,
        /// List of real bit masks to set the address floating bits for every combination (via bitwise OR).
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
        fn apply_mask(&self, memory: &Assignment) -> Vec<Assignment> {
            self.set_masks
                .iter()
                .map(|sm| Assignment {
                    address: (memory.address & self.reset_mask) | sm,
                    value: memory.value,
                })
                .collect()
        }
    }

    /// Program, which can be parsed from text input.
    #[derive(Debug)]
    pub struct Program {
        /// List of operations to perform.
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
        /// Execute the program for a particular chip version and return the sum
        /// of the memory values after completion.
        pub fn execute<M: Mask>(&self) -> u64 {
            // Current bit mask
            let mut mask = M::default();
            // Memory address to value map
            let mut memory: HashMap<u64, u64> = HashMap::new();
            for op in self.operations.iter() {
                match op {
                    Operation::SetMask(mv) => mask = mv[..].into(),
                    Operation::SetMemory(m) => {
                        for mem in mask.apply_mask(m) {
                            memory.insert(mem.address, mem.value);
                        }
                    }
                }
            }
            memory.values().sum()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 14,
    name: "Docking Data",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let program: Program = input.expect_input()?.parse()?;

            // Process
            Ok(program.execute::<MaskV1>().into())
        },
        // Part two
        |input| {
            // Generation
            let program: Program = input.expect_input()?.parse()?;

            // Process
            Ok(program.execute::<MaskV2>().into())
        },
    ],
};
