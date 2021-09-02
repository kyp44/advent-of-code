use std::{convert::TryInto, str::FromStr};

use itertools::Itertools;

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(654), Unsigned(57)],
    "20
15
10
5
5",
    vec![0u64, 0].answer_vec()
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
    fn combinations(&self, amount: u16) -> impl Iterator<Item = Vec<u16>> + '_ {
        (1..=self.containers.len())
            .map(move |k| {
                self.containers
                    .iter()
                    .combinations(k)
                    .map(|c| c.into_iter().copied().collect())
            })
            .flatten()
            .filter(move |c: &Vec<u16>| c.iter().sum::<u16>() == amount)
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
            /*for c in problem.combinations(25) {
                println!("{:?}", c);
            }*/
            Ok(Answer::Unsigned(
                problem.combinations(150).count().try_into().unwrap(),
            ))
        },
        // Part b)
        |input| {
            // Generation
            let problem: Problem = input.parse()?;

            // Process
            let combs: Vec<Vec<u16>> = problem.combinations(150).collect();
            let min = combs.iter().map(|cv| cv.len()).min().unwrap_or(0);
            let ans: u64 = combs.iter().filter_count(|cv| cv.len() == min);

            Ok(ans.into())
        },
    ],
};
