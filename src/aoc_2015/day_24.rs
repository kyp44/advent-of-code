use std::cmp::Ordering;

use crate::aoc::prelude::*;
use itertools::Itertools;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(10723906903), Unsigned(74850409)],
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
    vec![99u64, 44].answer_vec()
    }
}

struct Problem {
    weights: Vec<u32>,
    groups: usize,
    group_weight: u32,
}

impl Problem {
    fn from_str(groups: usize, s: &str) -> AocResult<Self> {
        let weights = u32::gather(s.lines())?;

        // Verify that the packages can be split into three compartments
        let sum: u32 = weights.iter().sum();
        if sum < 1 || usize::try_from(sum).unwrap() % groups != 0 {
            return Err(AocError::Process(format!("The weights have a sum of {}, and so cannot be split evenly into {} even compartments", sum, groups).into()));
        }

        Ok(Problem {
            weights,
            groups,
            group_weight: sum / u32::try_from(groups).unwrap(),
        })
    }

    fn solve(&self) -> AocResult<u64> {
        fn partitions(
            mut items: Vec<u32>,
            num_sets: usize,
            sum: u32,
        ) -> impl Iterator<Item = Vec<Vec<u32>>> {
            let mut parts = Vec::new();
            items.sort_unstable();
            match num_sets.cmp(&1) {
                Ordering::Equal => parts.push(vec![items]),
                Ordering::Greater => {
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
                                // Run recursively to ensure that the remaining part can be divided with matching sums
                                if let Some(mut part) = partitions(v2, num_sets - 1, sum).next() {
                                    part.insert(0, v1.clone());
                                    parts.push(part);
                                }
                            }
                        }
                        if !parts.is_empty() {
                            break;
                        }
                    }
                }
                _ => {}
            }
            parts.into_iter()
        }

        partitions(self.weights.clone(), self.groups, self.group_weight)
            .map(|parts| parts[0].iter().map(|x| u64::from(*x)).product())
            .min()
            .ok_or_else(|| AocError::Process("No solution found!".into()))
    }
}

pub const SOLUTION: Solution = Solution {
    day: 24,
    name: "It Hangs in the Balance",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let problem = Problem::from_str(3, input.expect_input()?)?;

            // Process
            Ok(problem.solve()?.into())
        },
        // Part b)
        |input| {
            // Generation
            let problem = Problem::from_str(4, input.expect_input()?)?;

            // Process
            Ok(problem.solve()?.into())
        },
    ],
};
