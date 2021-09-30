use itertools::Itertools;
use std::{collections::HashSet, iter::repeat, str::FromStr};

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
        if sum < 1 || sum % 3 != 0 {
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
        fn partitions(
            mut items: Vec<u32>,
            num_sets: usize,
            sum: u32,
        ) -> impl Iterator<Item = Vec<Vec<u32>>> {
            let mut parts = Vec::new();
            items.sort_unstable();
            if num_sets == 1 {
                parts.push(vec![items]);
            } else if num_sets > 1 {
                for size in 1..=(items.len() - (num_sets - 1)) {
                    // Is a set of this size always going to have a sum that is too large?
                    if items[..size].iter().sum::<u32>() > sum {
                        break;
                    }

                    // Now go through all sets of this size with the correct sum
                    for mut set in items
                        .iter()
                        .combinations(size)
                        .filter(|set| set.iter().copied().sum::<u32>() == sum)
                    {
                        // Now separate out the vector into the two vectors
                        let mut v1 = Vec::new();
                        let mut v2 = Vec::new();
                        for p in items.iter() {
                            match set.iter().position(|x| *x == p) {
                                None => v2.push(*p),
                                Some(i) => {
                                    v1.push(*p);
                                    set.remove(i);
                                }
                            }
                        }
                        if num_sets == 2 {
                            if v2.iter().sum::<u32>() == sum {
                                parts.push(vec![v1, v2]);
                            }
                        } else {
                            // Run recursively to ensure that the remaining part can be divided
                            if let Some(mut part) = partitions(v2, num_sets - 1, sum).next() {
                                part.insert(0, v1.clone());
                                if num_sets == 3 {
                                    println!("{:?}", part);
                                }

                                parts.push(part);
                            }
                        }
                    }
                    if !parts.is_empty() {
                        break;
                    }
                }
            }
            parts.into_iter()
        }

        let mut count = 0;
        for part in partitions(self.weights.clone(), 3, self.group_weight) {
            println!("{:?}", part);
            count += 1;
        }
        println!("TODO {}", count);
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
