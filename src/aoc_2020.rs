mod day_01;
mod day_02;
mod day_03;
mod day_04;
mod day_05;
mod day_06;
mod day_07;

use super::aoc::YearSolutions;

/// All of the solutions
pub const YEAR_SOLUTIONS: YearSolutions = YearSolutions {
    year: 2020,
    solutions: &[
        day_01::SOLUTION,
        day_02::SOLUTION,
        day_03::SOLUTION,
        day_04::SOLUTION,
        day_05::SOLUTION,
        day_06::SOLUTION,
        day_07::SOLUTION,
    ],
};
