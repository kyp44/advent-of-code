use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "1721
979
366
299
675
1456
";
            answers = unsigned![514579, 241861950];
        }
        actual_answers = unsigned![63616, 67877784];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use itertools::Itertools;

    /// Numeric type for expenses.
    pub type Expense = u32;

    /// Solves a part of the problem.
    pub fn solve(expenses: &[Expense], num_values: usize) -> AocResult<Answer> {
        let mut combinations = expenses.iter().combinations(num_values);
        loop {
            match combinations.next() {
                Some(v) => {
                    if v.iter().copied().sum::<u32>() == 2020 {
                        break Ok(Answer::Unsigned(v.iter().copied().product::<u32>().into()));
                    }
                }
                None => {
                    break Err(AocError::Process(
                        "No {num_values} values add to 2020".into(),
                    ))
                }
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 1,
    name: "Report Repair",
    preprocessor: Some(|input| Ok(Box::new(Expense::gather(input.lines())?).into())),
    solvers: &[
        // Part one
        |input| {
            // Processing
            solve(input.expect_data::<Vec<Expense>>()?, 2)
        },
        // Part two
        |input| {
            // Processing
            solve(input.expect_data::<Vec<Expense>>()?, 3)
        },
    ],
};
