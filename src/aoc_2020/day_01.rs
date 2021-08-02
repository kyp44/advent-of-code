use crate::aoc::prelude::*;
use itertools::Itertools;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Number;

    solution_test! {
    vec![Number(63616), Number(67877784)],
        "1721
979
366
299
675
1456
",
        vec![514579, 241861950].answer_vec()
    }
}

type Expense = u32;

pub const SOLUTION: Solution = Solution {
    day: 1,
    name: "Report Repair",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let values = Expense::gather(input.lines())?;

            // Processing
            let mut i = values.iter().combinations(2);
            loop {
                match i.next() {
                    Some(v) => {
                        if v[0] + v[1] == 2020 {
                            break Ok(Answer::Number((v[0] * v[1]).into()));
                        }
                    }
                    None => break Err(AocError::Process("No two values add to 2020".to_string())),
                }
            }
        },
        // Part b)
        |input| {
            // Generation
            let values = Expense::gather(input.lines())?;

            let mut i = values.iter().combinations(3);
            loop {
                match i.next() {
                    Some(v) if v[0] + v[1] + v[2] == 2020 => {
                        break Ok(Answer::Number((v[0] * v[1] * v[2]).into()));
                    }
                    None => {
                        break Err(AocError::Process("No three values add to 2020".to_string()));
                    }
                    _ => (),
                }
            }
        },
    ],
};
