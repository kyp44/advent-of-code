use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "3,4,3,1,2";
            answers = unsigned![5934, 26984457539];
        }
        actual_answers = unsigned![380612, 1710166656900];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;

    /// The maximum timer value, which is the number of days that
    /// new baby fish must wait to spawn for the first time.
    const MAX_TIME: usize = 8;

    /// A simulation of lanternfish population growth, which can be
    /// parsed from CSV text input of the initial fish timer values.
    ///
    /// This implements [`Iterator`] to carry out each step of the simulation,
    /// which yields the total number of fish in the population at each step.
    pub struct Simulation {
        /// Array of the number of fish with a certain time left until spawn.
        ///
        /// The index is the timer value and the array values are the number of
        /// fish with that timer value. This is a much more memory
        /// efficient way of handling the simulation that doesn't
        /// exhibit exponential memory growth.
        fish_with_time: [u64; MAX_TIME + 1],
    }
    impl FromStr for Simulation {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut fish = [0; MAX_TIME + 1];
            for timer in usize::from_csv(s)? {
                if timer > MAX_TIME {
                    return Err(AocError::InvalidInput(
                        format!("A timer of {timer} is not allowed!").into(),
                    ));
                }
                fish[timer] += 1;
            }
            Ok(Simulation {
                fish_with_time: fish,
            })
        }
    }
    impl Iterator for Simulation {
        type Item = u64;

        fn next(&mut self) -> Option<Self::Item> {
            let total_fish = self.fish_with_time.iter().sum();
            let num_spawn = self.fish_with_time[0];
            // Decrement the timers of each fish
            for i in 1..=MAX_TIME {
                self.fish_with_time[i - 1] = self.fish_with_time[i];
            }
            self.fish_with_time[MAX_TIME] = num_spawn;
            self.fish_with_time[6] += num_spawn;
            Some(total_fish)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 6,
    name: "Lanternfish",
    // NOTE: Simulation is an iterator so needs mutated, so we just parse it in each part.
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation and process
            Ok(Simulation::from_str(input.expect_text()?)?
                .nth(80)
                .unwrap()
                .into())
        },
        // Part two
        |input| {
            // Generation and process
            Ok(Simulation::from_str(input.expect_text()?)?
                .nth(256)
                .unwrap()
                .into())
        },
    ],
};
