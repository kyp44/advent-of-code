use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
            vec![Unsigned(6421487)],
        "5764801
    17807724",
    vec![14897079u64].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use derive_new::new;

    /// An [`Iterator`] that performs transformations on a number given the subject number.
    #[derive(new)]
    struct Transform {
        /// The subject number.
        subject: u64,
        /// The current value.
        #[new(value = "1")]
        value: u64,
    }
    impl Transform {
        /// Performs the transformation for the given loop size.
        fn transform(mut self, loop_size: usize) -> u64 {
            self.nth(loop_size - 1).unwrap()
        }
    }
    impl Iterator for Transform {
        type Item = u64;

        fn next(&mut self) -> Option<Self::Item> {
            self.value *= self.subject;
            self.value %= 20201227;
            Some(self.value)
        }
    }

    /// Problem definition, which can be parsed from text input.
    #[derive(Debug)]
    pub struct Problem {
        /// The public key for the card.
        card_key: u64,
        /// The public key for the door.
        door_key: u64,
    }
    impl FromStr for Problem {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let v = u64::gather(s.lines())?;

            if v.len() != 2 {
                return Err(AocError::InvalidInput(
                    format!(
                        "Input must contain exactly two numbers but found {}",
                        v.len(),
                    )
                    .into(),
                ));
            }

            Ok(Problem {
                card_key: v[0],
                door_key: v[1],
            })
        }
    }
    impl Problem {
        /// Solves the problem by using the public keys to find the loop sizes,
        /// and then using one of these to calculate the shared encryption key
        /// using the other's public key.
        pub fn solve(&self) -> AocResult<u64> {
            /// Internal function for [`Problem::solve`] that determines
            /// the secret loop size given an end key.
            fn find_loop_size(key: u64) -> usize {
                Transform::new(7).take_while(|v| *v != key).count() + 1
            }

            let card_loop = find_loop_size(self.card_key);
            let door_loop = find_loop_size(self.door_key);

            let enc_key = Transform::new(self.door_key).transform(card_loop);
            if enc_key != Transform::new(self.card_key).transform(door_loop) {
                return Err(AocError::Process(
                    "The encryption keys do not match!".into(),
                ));
            }

            Ok(enc_key)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 25,
    name: "Combo Breaker",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let problem: Problem = input.expect_input()?.parse()?;

            // Process
            Ok(problem.solve()?.into())
        },
    ],
};
