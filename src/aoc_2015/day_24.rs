use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
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

/// Contains solution implementation items.
mod solution {
    use aoc::prelude::*;
    use itertools::Itertools;
    use std::cmp::Ordering;

    /// Defines the problem, which can be parsed from text input.
    pub struct Problem {
        /// Weights of all the packages.
        package_weights: Vec<u32>,
        /// Number of groups/compartments into which to divide the packages.
        groups: usize,
        /// The total weight that each group/compartment must have for them to be equal.
        group_weight: u32,
    }
    impl Problem {
        /// For a given number of division groups/compartments, parse the package weight list from text input.
        pub fn from_str(groups: usize, s: &str) -> AocResult<Self> {
            let weights = u32::gather(s.lines())?;

            // Verify that the packages can be split into three compartments
            let sum: u32 = weights.iter().sum();
            if sum < 1 || usize::try_from(sum).unwrap() % groups != 0 {
                return Err(AocError::Process(format!("The weights have a sum of {sum}, and so cannot be split evenly into {groups} even compartments").into()));
            }

            Ok(Problem {
                package_weights: weights,
                groups,
                group_weight: sum / u32::try_from(groups).unwrap(),
            })
        }

        /// Solves a part of the problem by going through all possible partition of the packages
        /// into the compartment groups and finding the quantum entanglement of the group with the
        /// minimum number of packages (and also the minimal QE in the event of a tie).
        pub fn solve(&self) -> AocResult<u64> {
            /// Recursive sub-function of [`Problem::solve`] that returns an [`Iterator`] over all
            /// possible partitions of a set of numbers for a given number of sub sets.
            ///
            /// Also ensures that all of the subsets have a sum of `subset_sum`.
            fn partitions(
                mut items: Vec<u32>,
                num_subsets: usize,
                subset_sum: u32,
            ) -> impl Iterator<Item = Vec<Vec<u32>>> {
                let mut parts = Vec::new();
                items.sort_unstable();
                match num_subsets.cmp(&1) {
                    Ordering::Equal => parts.push(vec![items]),
                    Ordering::Greater => {
                        for size in 1..=(items.len() - (num_subsets - 1)) {
                            // Is a set of this size always going to have a sum that is too large?
                            if items[..size].iter().sum::<u32>() > subset_sum {
                                break;
                            }

                            // Now go through all sets of this size with the correct sum
                            for mut set in items
                                .iter()
                                .combinations(size)
                                .filter(|set| set.iter().copied().sum::<u32>() == subset_sum)
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
                                if num_subsets == 2 {
                                    if v2.iter().sum::<u32>() == subset_sum {
                                        parts.push(vec![v1, v2]);
                                    }
                                } else {
                                    // Run recursively to ensure that the remaining part can be divided with matching sums
                                    if let Some(mut part) =
                                        partitions(v2, num_subsets - 1, subset_sum).next()
                                    {
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

            partitions(self.package_weights.clone(), self.groups, self.group_weight)
                .map(|parts| parts[0].iter().map(|x| u64::from(*x)).product())
                .min()
                .ok_or(AocError::NoSolution)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 24,
    name: "It Hangs in the Balance",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let problem = Problem::from_str(3, input.expect_input()?)?;

            // Process
            Ok(problem.solve()?.into())
        },
        // Part two
        |input| {
            // Generation
            let problem = Problem::from_str(4, input.expect_input()?)?;

            // Process
            Ok(problem.solve()?.into())
        },
    ],
};
