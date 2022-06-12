use std::str::FromStr;

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(380612), Unsigned(1710166656900)],
    "3,4,3,1,2",
    vec![5934u64, 26984457539].answer_vec()
    }
}

const MAX_TIME: usize = 8;

struct Simulation {
    // Index is the timer value and the value is the number of fish with that timer
    fish: [u64; MAX_TIME + 1],
}
impl FromStr for Simulation {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fish = [0; MAX_TIME + 1];
        for timer in usize::from_csv(s)? {
            if timer > MAX_TIME {
                return Err(AocError::InvalidInput(
                    format!("A timer of {} is not allowed!", timer).into(),
                ));
            }
            fish[timer] += 1;
        }
        Ok(Simulation { fish })
    }
}
impl Iterator for Simulation {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let total_fish = self.fish.iter().sum();
        let num_spawn = self.fish[0];
        // Decrement the timers of each fish
        for i in 1..=MAX_TIME {
            self.fish[i - 1] = self.fish[i];
        }
        self.fish[MAX_TIME] = num_spawn;
        self.fish[6] += num_spawn;
        Some(total_fish)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 6,
    name: "Lanternfish",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation and process
            Ok(Simulation::from_str(input.expect_input()?)?
                .nth(80)
                .unwrap()
                .into())
        },
        // Part b)
        |input| {
            // Generation and process
            Ok(Simulation::from_str(input.expect_input()?)?
                .nth(256)
                .unwrap()
                .into())
        },
    ],
};
