use std::str::FromStr;

use itertools::{Itertools, MinMaxResult};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(341534), Unsigned(93397632)],
    "16,1,2,0,4,2,7,1,2,14",
    vec![37u64, 168].answer_vec()
    }
}

trait Part {
    fn fuel_used(dist: u64) -> u64;
}
struct PartA {}
impl Part for PartA {
    fn fuel_used(dist: u64) -> u64 {
        dist
    }
}
struct PartB {}
impl Part for PartB {
    fn fuel_used(dist: u64) -> u64 {
        dist * (dist + 1) / 2
    }
}

struct CrabSubs {
    positions: Box<[u64]>,
}
impl FromStr for CrabSubs {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CrabSubs {
            positions: u64::from_csv(s)?.into_boxed_slice(),
        })
    }
}
impl CrabSubs {
    fn align<P: Part>(&self) -> AocResult<u64> {
        match self.positions.iter().minmax() {
            MinMaxResult::MinMax(min, max) => Ok(((*min)..=(*max))
                .map(|p| {
                    self.positions
                        .iter()
                        .map(|x| P::fuel_used(x.abs_diff(p)))
                        .sum()
                })
                .min()
                .unwrap()),
            MinMaxResult::OneElement(v) => Ok(*v),
            _ => Err(AocError::Process("Data empty!".into())),
        }
    }
}

pub const SOLUTION: Solution = Solution {
    day: 7,
    name: "The Treachery of Whales",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let subs = CrabSubs::from_str(input)?;

            // Process
            Ok(subs.align::<PartA>()?.into())
        },
        // Part b)
        |input| {
            // Generation
            let subs = CrabSubs::from_str(input)?;

            // Process
            Ok(subs.align::<PartB>()?.into())
        },
    ],
};
