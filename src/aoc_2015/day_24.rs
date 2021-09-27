use std::{iter::repeat, str::FromStr};

use itertools::Itertools;
use partitions::PartitionVec;

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![],
    "1
2
3
4
5
7
8
9
10
11",
    vec![1u64].answer_vec()
    }
}

struct Problem {
    weights: Vec<u32>,
    group_weight: u32,
}
impl FromStr for Problem {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let weights = u32::gather(s.lines())?;

        // Verify that the packages can be split into three compartments
        let sum: u32 = weights.iter().sum();
        if sum < 1 && sum % 3 != 0 {
            return Err(AocError::Process(format!("The weights have a sum of {}, and so cannot be split evenly into three compartments", sum).into()));
        }

        Ok(Problem {
            weights,
            group_weight: sum / 3,
        })
    }
}
impl Problem {
    fn solve(&self) -> AocResult<u64> {
        let sum: u32 = self.weights.iter().sum();
        assert!(sum % 2 == 0);
        let size = sum / 2;

        #[derive(new, Clone)]
        struct BoolIter {
            #[new(value = "Some(false)")]
            value: Option<bool>,
        }
        impl Iterator for BoolIter {
            type Item = bool;

            fn next(&mut self) -> Option<Self::Item> {
                match self.value {
                    None => self.value,
                    Some(b) => {
                        if b {
                            self.value = None;
                            Some(true)
                        } else {
                            self.value = Some(true);
                            Some(false)
                        }
                    }
                }
            }
        }

        fn binary_partitions(items: &[u32]) -> impl Iterator<Item = PartitionVec<u32>> + '_ {
            repeat(BoolIter::new())
                .take(items.len() - 1)
                .multi_cartesian_product()
                .filter(|gv| gv.iter().any(|b| !*b))
                .map(|gv| {
                    let mut partition: PartitionVec<u32> = items.iter().collect();
                    let mut index = None;
                    for (i, group) in gv.into_iter().enumerate() {
                        if group {
                            partition.union(0, i + 1);
                        } else {
                            match index {
                                None => {
                                    index = Some(i + 1);
                                }
                                Some(idx) => partition.union(idx, i + 1),
                            }
                        }
                    }
                    partition
                })
        }
        for bin_part in binary_partitions(&self.weights) {
            println!(
                "{}",
                bin_part.all_sets().map(|s| format!("{:?}", s)).join(", ")
            );
        }
        println!("TODO {}", binary_partitions(&self.weights).count());
        Ok(0)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 24,
    name: "It Hangs in the Balance",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let problem: Problem = input.parse()?;
            println!("TODO weights: {:?}", problem.weights);

            // Process
            Ok(problem.solve()?.into())
        },
    ],
};
