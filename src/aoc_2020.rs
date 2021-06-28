mod day_01;
mod day_02;
mod day_03;
mod day_04;
mod day_05;
mod day_06;
mod day_07;
mod day_08;
mod day_09;
mod day_10;
mod day_11;
mod day_12;

use super::aoc::YearSolutions;
use lazy_static::lazy_static;

lazy_static! {
    /// All of the solutions
    pub static ref YEAR_SOLUTIONS: YearSolutions = YearSolutions {
        year: 2020,
        solutions: vec![
            day_01::SOLUTION,
            day_02::SOLUTION,
            day_03::SOLUTION,
            day_04::SOLUTION,
            day_05::SOLUTION,
            day_06::SOLUTION,
            day_07::SOLUTION,
            day_08::SOLUTION,
            day_09::SOLUTION,
            day_10::SOLUTION,
            day_11::SOLUTION,
        day_12::SOLUTION,
        ],
    };
}
