use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "1
2
-3
3
-2
0
4";
            answers = signed![3, 1623178306];
        }
        actual_answers = signed![15297, 2897373276210];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::circular_list::{CircularList, DoublyLinked};
    use nom::Finish;

    /// The file that needs to be decrypted, which can be parsed
    /// from text input.
    pub struct File {
        /// The numerical data comprising the file.
        data: Vec<i16>,
    }
    impl FromStr for File {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let data = s
                .lines()
                .map(|line| {
                    nom::character::complete::i16::<_, NomParseError>(line)
                        .finish()
                        .discard_input()
                })
                .collect::<Result<Vec<_>, _>>()?;

            if data.is_empty() {
                return Err(AocError::InvalidInput("Input contains no numbers!".into()));
            }

            Ok(Self { data })
        }
    }
    impl File {
        /// Decrypts the file and returns the sum of the grove coordinates.
        ///
        /// Decryption is performed according to the part `P` decryption
        /// processor.
        pub fn grove_coordinate_sum<P: Part>(&self) -> i64 {
            P::new(self).decrypt()
        }
    }

    /// Behavior specific to a particular part of the problem.
    pub trait Part {
        /// Creates a new decryption processor for the given `file`.
        fn new(file: &File) -> Self;

        /// Returns the circular buffer containing the file data.
        fn buffer(&self) -> &CircularList<DoublyLinked<i64>>;

        /// Decrypts the file and returns the sum of the grove coordinates.
        fn decrypt(&self) -> i64;

        /// Determines and returns the sum of the grove coordinates
        /// once mixing has been completed.
        fn grove_coordinates_sum(&self) -> i64 {
            let zero_node = self
                .buffer()
                .iter_const()
                .find(|n| *n.value() == 0)
                .unwrap();

            (1..=3).map(|i| zero_node.node_at(1000 * i).value()).sum()
        }

        /// Mixes the all of the numbers *once* according to the mixing rules.
        fn mix(&self) {
            for mut node in self.buffer().iter_const() {
                let shift = (*node.value()).try_into().unwrap();

                node.shift(shift);
            }
        }
    }

    /// The decryption processor for part one.
    pub struct PartOne {
        /// The circular list of the file data.
        buffer: CircularList<DoublyLinked<i64>>,
    }
    impl Part for PartOne {
        fn new(file: &File) -> Self {
            Self {
                buffer: CircularList::new(file.data.iter().map(|v| i64::from(*v))),
            }
        }

        fn buffer(&self) -> &CircularList<DoublyLinked<i64>> {
            &self.buffer
        }

        fn decrypt(&self) -> i64 {
            self.mix();

            self.grove_coordinates_sum()
        }
    }

    /// The decryption key for part two.
    const DECRYPTION_KEY: i64 = 811589153;

    /// The decryption processor for part two.
    pub struct PartTwo {
        /// The circular list of the file data.
        buffer: CircularList<DoublyLinked<i64>>,
    }
    impl Part for PartTwo {
        fn new(file: &File) -> Self {
            Self {
                buffer: CircularList::new(file.data.iter().map(|v| DECRYPTION_KEY * i64::from(*v))),
            }
        }

        fn buffer(&self) -> &CircularList<DoublyLinked<i64>> {
            &self.buffer
        }

        fn decrypt(&self) -> i64 {
            for _ in 0..10 {
                self.mix();
            }

            self.grove_coordinates_sum()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 20,
    name: "Grove Positioning System",
    preprocessor: Some(|input| Ok(Box::new(File::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<File>()?
                .grove_coordinate_sum::<PartOne>()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<File>()?
                .grove_coordinate_sum::<PartTwo>()
                .into())
        },
    ],
};
