use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "mjqjpqmgbljsphdztnvjfqwrcgsmlb";
            answers = unsigned![7, 19];
        }
        example {
            input = "bvwbjplbgvbhsrlpgdmjqwftvncz";
            answers = unsigned![5, 23];
        }
        example {
            input = "nppdvjthqldpwncqszvftbrmjlhg";
            answers = unsigned![6, 23];
        }
        example {
            input = "nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg";
            answers = unsigned![10, 29];
        }
        example {
            input = "zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw";
            answers = unsigned![11, 26];
        }
        actual_answers = unsigned![1804, 2508];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use std::collections::HashSet;

    /// The data stream sent from the elves to the device.
    pub struct Datastream {
        /// The buffer of data sent.
        buffer: Vec<char>,
    }
    impl From<&str> for Datastream {
        fn from(value: &str) -> Self {
            Self {
                buffer: value.chars().collect(),
            }
        }
    }
    impl Datastream {
        /// Returns the index of the last character sent (where the first characteris 1, not 0)
        /// such that the last `size` characters are all distinct.
        fn distinct_window(&self, size: usize) -> AocResult<usize> {
            for (idx, chunk) in self.buffer.windows(size).enumerate() {
                let set: HashSet<char> = HashSet::from_iter(chunk.iter().copied());
                if set.len() == size {
                    // We want the index of the last character, not the first
                    return Ok(idx + size);
                }
            }

            Err(AocError::NoSolution)
        }

        /// Returns the index of the last character sent such that the
        /// start-of-packet marker (for part one) has been sent.
        ///
        /// The start-of-packet marker is a consecutive sequence of 4
        /// distinct characters.
        pub fn start_marker(&self) -> AocResult<usize> {
            self.distinct_window(4)
        }

        /// Returns the index of the last character sent such that the
        /// start-of-message marker (for part two) has been sent.
        ///
        /// The start-of-message marker is a consecutive sequence of 14
        /// distinct characters.
        pub fn message_marker(&self) -> AocResult<usize> {
            self.distinct_window(14)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 6,
    name: "Tuning Trouble",
    preprocessor: Some(|input| Ok(Box::new(Datastream::from(input)).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(
                u64::try_from(input.expect_data::<Datastream>()?.start_marker()?)
                    .unwrap()
                    .into(),
            )
        },
        // Part two
        |input| {
            // Process
            Ok(
                u64::try_from(input.expect_data::<Datastream>()?.message_marker()?)
                    .unwrap()
                    .into(),
            )
        },
    ],
};
