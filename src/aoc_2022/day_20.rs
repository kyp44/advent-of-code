use aoc::prelude::*;
use std::str::FromStr;

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
    use aoc::circular_list::CircularList;
    use nom::Finish;

    pub struct File {
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
        pub fn grove_coordinate_sum<P: Part>(&self) -> i64 {
            P::new(&self).solve()
        }
    }

    pub trait Part {
        fn new(file: &File) -> Self;
        fn buffer(&self) -> &CircularList<i64>;

        fn solve(&self) -> i64;

        fn grove_coordinates_sum(&self) -> i64 {
            let zero_node = self
                .buffer()
                .iter_const()
                .find(|n| *n.value() == 0)
                .unwrap();

            (1..=3).map(|i| zero_node.node_at(1000 * i).value()).sum()
        }

        fn mix(&self) {
            for mut node in self.buffer().iter_const() {
                let shift = (*node.value()).try_into().unwrap();

                node.shift(shift);
            }
        }
    }

    pub struct PartOne {
        buffer: CircularList<i64>,
    }
    impl Part for PartOne {
        fn new(file: &File) -> Self {
            Self {
                buffer: CircularList::new(file.data.iter().map(|v| i64::from(*v))).unwrap(),
            }
        }

        fn buffer(&self) -> &CircularList<i64> {
            &self.buffer
        }

        fn solve(&self) -> i64 {
            self.mix();

            self.grove_coordinates_sum()
        }
    }

    const DECRYPTION_KEY: i64 = 811589153;

    pub struct PartTwo {
        buffer: CircularList<i64>,
    }
    impl Part for PartTwo {
        fn new(file: &File) -> Self {
            Self {
                buffer: CircularList::new(file.data.iter().map(|v| DECRYPTION_KEY * i64::from(*v)))
                    .unwrap(),
            }
        }

        fn buffer(&self) -> &CircularList<i64> {
            &self.buffer
        }

        fn solve(&self) -> i64 {
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
