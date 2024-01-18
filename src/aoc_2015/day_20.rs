use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "5000";
            answers = unsigned![180, 168];
        }
        actual_answers = unsigned![786240, 831600];
    }
}

/// Contains solution implementation items.
mod solution {
    use num::Integer;

    use super::*;

    /// Represents the number of presents delivered to each house below some maximum house number.
    pub struct Delivery {
        /// The houses of interest are those that get at least this number of presents.
        target: usize,
        /// The number of presents delivered to each house.
        presents: Vec<usize>,
    }
    impl Delivery {
        /// Creates a delivery base on a target number, a present multiplier (which is the number of
        /// presents each elf delivers divided by its elf number), and an optional maximum
        /// number of houses to which each elf delivers.
        pub fn new(target: usize, present_mult: usize, house_limit: Option<usize>) -> Self {
            // Maximum house for which we need to compute the number of presents
            // as this is guaranteed to be above the target.
            // This was derived from the lowest even house number n such that:
            // target <= 10*(n + n/2)
            // since n/2 will be a divisor for even n.
            let mut max = 2 * target / 30;
            max += if max.is_even() { 2 } else { 1 };

            // We implement a seive that calculates all number of presents
            // (effectively the sum of divisors) for all numbers up to max.
            let mut presents = vec![0; max];
            // Each elf
            for i in 1..=max {
                let mut count = 0;
                let mut j = i;
                loop {
                    presents[j - 1] += present_mult * i;
                    j += i;
                    if j > max {
                        break;
                    }
                    count += 1;
                    if let Some(l) = house_limit {
                        if count > l {
                            break;
                        }
                    }
                }
            }

            Delivery { target, presents }
        }

        /// Returns the first house number who got at least as many presents as the target number.
        pub fn first_house(&self) -> AocResult<u64> {
            Ok((self
                .presents
                .iter()
                .position(|p| *p >= self.target)
                .ok_or_else(|| AocError::Process("No solution found!".into()))?
                + 1)
            .try_into()
            .unwrap())
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 20,
    name: "Infinite Elves and Infinite Houses",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let delivery = Delivery::new(usize::from_str(input.expect_input()?)?, 10, None);

            // Process
            /*for h in 1..10 {
                println!("House {}: {}", h, delivery.presents[h - 1]);
            }*/

            Ok(delivery.first_house()?.into())
        },
        // Part two
        |input| {
            // Generation
            let delivery = Delivery::new(usize::from_str(input.expect_input()?)?, 11, Some(50));

            // Process
            /*for h in 1..30 {
                println!("House {}: {}", h, delivery.presents[h - 1]);
            }*/

            Ok(delivery.first_house()?.into())
        },
    ],
};
