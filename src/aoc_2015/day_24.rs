use std::{collections::HashSet, iter::repeat, str::FromStr};

use itertools::Itertools;

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

        fn partitions(
            mut items: Vec<u32>,
            num_sets: usize,
            sum: u32,
        ) -> impl Iterator<Item = Vec<Vec<u32>>> {
            let mut parts = HashSet::new();
            items.sort_unstable();

            if num_sets == 1 {
                parts.insert(vec![items]);
            } else if num_sets > 1 {
                for (v1, v2) in repeat(BoolIter::new())
                    .take(items.len() - 1)
                    .multi_cartesian_product()
                    .filter(|gv| !gv.iter().all(|b| *b))
                    .map(move |gv| {
                        let mut v1 = vec![items[0]];
                        let mut v2 = Vec::new();
                        for (i, group) in gv.into_iter().enumerate() {
                            if group {
                                v1.push(items[i + 1])
                            } else {
                                v2.push(items[i + 1])
                            }
                        }
                        v1.sort_unstable();
                        v2.sort_unstable();

                        (v1, v2)
                    })
                {
                    if num_sets == 2 {
                        let mut v = vec![v1, v2];
                        v.sort_unstable();
                        parts.insert(v);
                    } else if v1.len() >= num_sets - 1 {
                        for mut v1s in partitions(v1, num_sets - 1, sum) {
                            v1s.push(v2.clone());
                            v1s.sort_unstable();
                            parts.insert(v1s);
                        }
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
