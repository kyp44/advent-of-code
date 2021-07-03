use super::super::aoc::{AocError, Parseable, Solution};
use itertools::Itertools;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![63616, 67877784],
        "1721
979
366
299
675
1456
",
        vec![Some(514579), Some(241861950)]
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
                            break Ok((v[0] * v[1]).into());
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
                        break Ok((v[0] * v[1] * v[2]).into());
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
