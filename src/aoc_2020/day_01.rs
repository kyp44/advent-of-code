use crate::aoc::prelude::*;
use itertools::Itertools;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(63616), Unsigned(67877784)],
        "1721
979
366
299
675
1456
",
        vec![514579u64, 241861950].answer_vec()
    }
}

type Expense = u32;

pub const SOLUTION: Solution = Solution {
    day: 1,
    name: "Report Repair",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let values = Expense::gather(input.expect_input()?.lines())?;

            // Processing
            let mut i = values.iter().combinations(2);
            loop {
                match i.next() {
                    Some(v) => {
                        if v[0] + v[1] == 2020 {
                            break Ok(Answer::Unsigned((v[0] * v[1]).into()));
                        }
                    }
                    None => break Err(AocError::Process("No two values add to 2020".into())),
                }
            }
        },
        // Part b)
        |input| {
            // Generation
            let values = Expense::gather(input.expect_input()?.lines())?;

            let mut i = values.iter().combinations(3);
            loop {
                match i.next() {
                    Some(v) if v[0] + v[1] + v[2] == 2020 => {
                        break Ok(Answer::Unsigned((v[0] * v[1] * v[2]).into()));
                    }
                    None => {
                        break Err(AocError::Process("No three values add to 2020".into()));
                    }
                    _ => (),
                }
            }
        },
    ],
};
