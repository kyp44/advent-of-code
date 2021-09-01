use std::{iter::Filter, slice::Iter, str::FromStr};

use itertools::Itertools;

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![],
    "20
15
10
5
5",
    Vec::new()
    }
}

struct Problem {
    containers: Box<[u16]>,
}
impl FromStr for Problem {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Problem {
            containers: u16::gather(s.lines())?.into_boxed_slice(),
        })
    }
}
impl Problem {
    fn combinations(amount: u16) {
        todo!()
    }
}

struct Combinations<'a, P> {
    elements: &'a [u16],
    i: u16,
    iter: Filter<itertools::Combinations<Iter<'a, u16>>, P>,
}
impl<'a, P> Combinations<'a, P>
where P: impl Fn(&Vec<&u16>) -> bool
{
    fn new(elements: &'a [u16], amount: u16) -> Self {
        Combinations {
            elements,
            i: 1,
            iter: elements
                .iter()
                .combinations(1)
                .filter(|x: &Vec<&u16>| false),
        }
    }
}
impl Iterator for Combinations<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.elements.iter();
        todo!()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 17,
    name: "No Such Thing as Too Much",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let problem: Problem = input.parse()?;

            // Process
            Ok(0u64.into())
        },
    ],
};
