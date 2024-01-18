use std::str::FromStr;

use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "vJrwpWtwJgWrhcsFMMfFFhFp
jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
PmmdzqPrVvPwwTWBwg
wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
ttgJtRGJQctTZtZT
CrZsJsPPZsGzwwsLwLmpwMDw";
            answers = unsigned![157, 70];
        }
        actual_answers = unsigned![7691, 2508];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use itertools::process_results;
    use std::ascii::Char as AsciiChar;
    use std::collections::HashSet;
    use std::str::FromStr;

    /// An extension trait for characters as an item type.
    trait CharExt {
        /// Returns the priority of the character as an item type.
        fn priority(&self) -> AocResult<u8>;
    }
    impl CharExt for char {
        fn priority(&self) -> AocResult<u8> {
            if !self.is_ascii_alphabetic() {
                return Err(AocError::Process(
                    format!("No priority defined for '{self}'").into(),
                ));
            }

            let ord = self.as_ascii().unwrap().to_u8();
            Ok(if self.is_ascii_lowercase() {
                ord - AsciiChar::SmallA.to_u8() + 1
            } else {
                ord - AsciiChar::CapitalA.to_u8() + 27
            })
        }
    }

    /// A rucksack consisting of two compartments.
    #[derive(Debug)]
    struct Rucksack {
        /// Item types in the first compartment.
        compartment_1: Vec<char>,
        /// Item types in the second compartment.
        compartment_2: Vec<char>,
    }
    impl FromStr for Rucksack {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // If we have an odd number of item types this is a problem
            if s.len() % 2 == 1 {
                return Err(AocError::InvalidInput(
                    "Rucksack has an odd number of item types!".into(),
                ));
            }

            let compartment_len = s.len() / 2;
            Ok(Self {
                compartment_1: s[..compartment_len].chars().collect(),
                compartment_2: s[compartment_len..].chars().collect(),
            })
        }
    }
    impl Rucksack {
        /// Calculates the priority of the singular item common to both
        /// compartments.
        pub fn common_priority(&self) -> AocResult<u8> {
            let compartment_1 = self.compartment_1.iter().copied().collect::<HashSet<_>>();
            let compartment_2 = self.compartment_2.iter().copied().collect::<HashSet<_>>();
            let common = compartment_1
                .intersection(&compartment_2)
                .collect::<Vec<_>>();

            if common.len() != 1 {
                Err(AocError::Process(
                    "The compartments do not have exactly one item type in common!".into(),
                ))
            } else {
                common[0].priority()
            }
        }

        /// Returns an iterator over all item types in the rucksack from
        /// both compartments.
        pub fn all_item_types(&self) -> impl Iterator<Item = char> + '_ {
            self.compartment_1
                .iter()
                .copied()
                .chain(self.compartment_2.iter().copied())
        }
    }

    /// A set of rucksacks.
    pub struct PackSet {
        /// The actual rucksacks.
        rucksacks: Vec<Rucksack>,
    }
    impl FromStr for PackSet {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                rucksacks: s
                    .lines()
                    .map(Rucksack::from_str)
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
    }
    impl PackSet {
        /// Determines the solution to part one.
        ///
        /// This is done by determining the common item type between the two
        /// compartments.
        /// The priorities of these common item types are then summed over all
        /// of the rucksacks.
        pub fn common_priority_sum(&self) -> AocResult<u64> {
            process_results(
                self.rucksacks
                    .iter()
                    .map(|r| r.common_priority().map(u64::from)),
                |iter| iter.sum::<u64>(),
            )
        }

        /// Determines the solution to part two.
        ///
        /// This is done by dividing the rucksacks into groups of three.
        /// For each group, the common item type is identified and its
        /// priority determined.
        /// These priorities are then summed over all the groups.
        pub fn badge_priority_sum(&self) -> AocResult<u64> {
            if self.rucksacks.len() % 3 != 0 {
                return Err(AocError::Process(
                    "The number of rucksacks is not divisible by 3!".into(),
                ));
            }

            process_results(
                self.rucksacks.iter().array_chunks::<3>().map(|sacks| {
                    let ints = sacks
                        .into_iter()
                        .map(|rs| rs.all_item_types().collect::<HashSet<_>>())
                        .reduce(|a, b| a.intersection(&b).copied().collect::<HashSet<_>>())
                        .unwrap();

                    if ints.len() != 1 {
                        Err(AocError::Process(
                            "A group does not have exactly one item type in common!".into(),
                        ))
                    } else {
                        Ok(u64::from(ints.into_iter().next().unwrap().priority()?))
                    }
                }),
                |iter| iter.sum(),
            )
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 3,
    name: "Rucksack Reorganization",
    preprocessor: Some(|input| Ok(Box::new(PackSet::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<PackSet>()?
                .common_priority_sum()?
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input.expect_data::<PackSet>()?.badge_priority_sum()?.into())
        },
    ],
};
