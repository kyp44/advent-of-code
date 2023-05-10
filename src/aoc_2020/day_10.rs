use std::str::FromStr;

use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "16
10
15
5
1
11
7
19
6
12
4";
            answers = vec![35u64, 8].answer_vec();
        }
        example {
            input = "28
33
18
42
31
14
46
20
48
47
24
23
49
45
19
38
39
11
1
32
25
35
8
17
7
9
4
2
34
10
3";
            answers = vec![220u64, 19208].answer_vec();
        }
        actual_answers = vec![Unsigned(2100), Unsigned(16198260678656)];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::{
        fmt,
        ops::{Add, Sub},
    };

    use super::*;

    /// An adapter with a particular output voltage.
    #[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
    struct Adapter {
        /// The output joltage of the adapter.
        output_joltage: u32,
    }
    impl fmt::Debug for Adapter {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.output_joltage.fmt(f)
        }
    }
    impl From<u32> for Adapter {
        fn from(value: u32) -> Self {
            Self {
                output_joltage: value,
            }
        }
    }
    impl Add for Adapter {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            (self.output_joltage + rhs.output_joltage).into()
        }
    }

    /// Difference between two adapters.
    #[derive(Debug)]
    enum AdapterDifference {
        /// Output joltage difference between adapters is too large.
        Incompatible,
        /// They are compatible with the joltage difference.
        Compatible(u32),
    }
    impl AdapterDifference {
        /// Returns whether the compared adapters are compatible or not.
        fn is_compatible(&self) -> bool {
            matches!(self, Self::Compatible(_))
        }
    }
    impl Sub for Adapter {
        type Output = AdapterDifference;

        // Note that this subtraction is commutative.
        fn sub(self, rhs: Self) -> Self::Output {
            let diff = self.output_joltage.abs_diff(rhs.output_joltage);
            if diff > 3 {
                AdapterDifference::Incompatible
            } else {
                AdapterDifference::Compatible(diff)
            }
        }
    }

    /// The complete set of adapters, which can be parsed from text input.
    pub struct AdapterSet {
        /// The set of adapters in order of increasing output joltage.
        ///
        /// Also includes the outlet (0 jolts) and the maximum built-in adapter
        /// input joltage.
        adapters: Vec<Adapter>,
    }
    impl FromStr for AdapterSet {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // Read in adapters.
            let mut adapters: Vec<_> = u32::gather(s.lines())?
                .into_iter()
                .map(Adapter::from)
                .collect();

            // Add the outlet and built-in adapter.
            adapters.push(0.into());
            adapters.push(*adapters.iter().max().unwrap() + 3.into());

            // Remove duplicates and sort
            adapters.dedup();
            adapters.sort_unstable();

            // Check that there are no gaps that are too large
            let adapters = Self { adapters };
            if adapters.differences().any(|d| !d.is_compatible()) {
                return Err(AocError::InvalidInput(
                    "The device cannot ever be powered because there is a gap of over 3 jolts"
                        .into(),
                ));
            }

            Ok(adapters)
        }
    }
    impl AdapterSet {
        /// Returns an [`Iterator`] over the difference between the ordered adapters/joltages.
        fn differences(&self) -> impl Iterator<Item = AdapterDifference> + '_ {
            self.adapters.windows(2).map(|w| w[1] - w[0])
        }

        /// Counts the number of adapter transitions that have a particular joltage difference.
        pub fn count_joltage_differences(&self, difference: u32) -> usize {
            // Verify that the sorted adapters are all compatible
            self.differences().filter_count(|diff| match diff {
                AdapterDifference::Incompatible => false,
                AdapterDifference::Compatible(d) => *d == difference,
            })
        }

        /// Counts the number of possible arrangements of the adapters.
        pub fn count_arrangements(&self) -> usize {
            // NOTE: We could theoretically use aoc::tree_search::GlobalStateTreeNode along with
            // the CountLeaves global state, but the tree is far to large so that the below special
            // algorithm is needed to solve in a reasonable time.

            // For each adapter we store the number of variations between it and the device
            // if we were to keep the adapter chain between it and the outlet.
            let mut variations: std::collections::HashMap<Adapter, usize> =
                std::collections::HashMap::new();
            // The previous recent number of variations
            let mut last_var = 1;

            // The algorithm here works work backwards just because it's more natural to take slices
            // forward rather than backward.
            for (i, v) in self.adapters.iter().enumerate().rev() {
                // Each new number of variations is then the sum of any potential number
                // of variations if there are adapters with any of the next three consecutive
                // output joltages, or the last variation if the the next gap is 3 jolts.
                let var = std::cmp::max(
                    self.adapters[i + 1..]
                        .iter()
                        .take_while(|vp| (**vp - *v).is_compatible())
                        .map(|vp| variations[vp])
                        .sum(),
                    last_var,
                );
                variations.insert(*v, var);
                last_var = var;
                //println!("{} {} {}", i, v, var);
            }
            last_var
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 10,
    name: "Adapter Array",
    preprocessor: Some(|input| Ok(Box::new(AdapterSet::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Processing
            let adapters = input.expect_data::<AdapterSet>()?;
            Ok(Answer::Unsigned(
                (adapters.count_joltage_differences(1) * adapters.count_joltage_differences(3))
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            // Processing
            Ok(Answer::Unsigned(
                input
                    .expect_data::<AdapterSet>()?
                    .count_arrangements()
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
