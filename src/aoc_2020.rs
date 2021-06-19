mod day_01;
mod day_02;
mod day_03;
mod day_04;
mod day_05;
mod day_06;
mod day_07;
mod day_08;

use super::aoc::YearSolutions;
use lazy_static::lazy_static;

lazy_static! {
    /// All of the solutions
    pub static ref YEAR_SOLUTIONS: YearSolutions = YearSolutions {
        year: 2020,
        solutions: vec![day_01::SOLUTION,
                        day_02::SOLUTION,
                        day_03::SOLUTION,
                        day_04::SOLUTION,
                        day_05::SOLUTION,
                        day_06::SOLUTION,
                        day_07::SOLUTION,
                        day_08::SOLUTION,
        ],
    };
}






