mod day_01;
mod day_02;
mod day_03;
mod day_04;

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
        ],
    };
}






