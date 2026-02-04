use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "1000
2000
3000

4000

5000
6000

7000
8000
9000

10000";
            answers = unsigned![24000, 45000];
        }
        actual_answers = unsigned![72718, 213089];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use nom::{character::complete::multispace1, combinator::map, multi::separated_list1};

    /// An elf carrying some food items.
    #[derive(Debug)]
    pub struct Elf {
        /// The calories of the food items.
        items: Vec<u32>,
    }
    impl Parsable<'_> for Elf {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_list1(multispace1, nom::character::complete::u32),
                |items| Self { items },
            )
            .parse(input)
        }
    }
    impl Elf {
        /// Calculates the total calories of all food items that the elf
        /// is carrying.
        pub fn total(&self) -> u32 {
            self.items.iter().sum()
        }
    }
}
use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 1,
    name: "Calorie Counting",
    preprocessor: Some(|input| Ok(Box::new(Elf::gather(input.split("\n\n"))?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Vec<Elf>>()?
                .iter()
                .map(|e| e.total())
                .max()
                .unwrap()
                .into())
        },
        // Part two
        |input| {
            // Process
            let mut totals: Vec<_> = input
                .expect_data::<Vec<Elf>>()?
                .iter()
                .map(|e| e.total())
                .collect();
            totals.sort();
            totals.reverse();

            Ok(totals.iter().take(3).sum::<u32>().into())
        },
    ],
};
