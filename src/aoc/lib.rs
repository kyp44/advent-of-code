#![feature(slice_pattern)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use anyhow::Context;
use colored::Colorize;
use itertools::Itertools;
use num::{Integer, Signed};
use parse::NomParseError;
use std::any::Any;
use std::borrow::Cow;
use std::fmt::Debug;
use std::fs;
use std::ops::RangeInclusive;
use thiserror::Error;

pub mod evolver;
pub mod grid;
pub mod iter;
pub mod parse;

// Prelude
pub mod prelude {
    pub use super::{
        char_add, evolver::Evolver, grid::CharGrid, grid::Grid, grid::GridDefault, grid::GridPoint,
        grid::GridSize, grid::GridSizeExt, grid::PointTryInto, iter::IteratorExt, iter::StrExt,
        parse::BitInput, parse::DiscardInput, parse::NomParseError, parse::NomParseResult,
        parse::Parseable, parse::Sections, Answer, AnswerVec, AocError, AocResult, RangeExt,
        Solution, SolverInput, VectorExt, YearSolutions,
    };
    pub use aoc_derive::CharGridDebug;
}

// Custom error type for AoC problem functions.
#[derive(Debug, Clone, Error)]
pub enum AocError {
    #[error("Year {0} is not yet solved")]
    NoYear(u32),
    #[error("Day {0} is not yet solved")]
    NoDay(u32),
    #[error("Day {0} is not in the range of {} to {}", .1.start(), .1.end())]
    DayRange(u32, RangeInclusive<u32>),
    #[error("Could not parse input")]
    NomParse(
        #[source]
        #[from]
        NomParseError,
    ),
    #[error("Invalid input: {0}")]
    InvalidInput(Cow<'static, str>),
    #[error("Error while processing: {0}")]
    Process(Cow<'static, str>),
    #[error("No solution found!")]
    NoSolution,
}
pub type AocResult<T> = Result<T, AocError>;

// TODO: Use some modules here to better organize these items.

// TODO: Do we want some kind of modulo arithmetic type, probably crates for this.
// In particular sometimes we want, modulo `n` but from `1` to `n` instead of `0` to `n-1`.

/// Extension methods for vector types.
pub trait VectorExt<T> {
    /// Calculate the Manhattan length of the vector.
    fn manhattan_len(&self) -> T;
}
impl<T> VectorExt<T> for cgmath::Vector2<T>
where
    T: Signed,
{
    fn manhattan_len(&self) -> T {
        self.x.abs() + self.y.abs()
    }
}
impl<T> VectorExt<T> for cgmath::Vector3<T>
where
    T: Signed,
{
    fn manhattan_len(&self) -> T {
        self.x.abs() + self.y.abs() + self.z.abs()
    }
}

// Extension trait for ranges.
pub trait RangeExt<T>: Sized {
    fn len(&self) -> T;
    fn intersection(&self, other: &Self) -> Option<Self>;
}
impl<T> RangeExt<T> for RangeInclusive<T>
where
    T: Integer + Copy,
{
    fn len(&self) -> T {
        *self.end() - *self.start() + T::one()
    }

    fn intersection(&self, other: &Self) -> Option<Self> {
        let range = *self.start().max(other.start())..=*self.end().min(other.end());
        if self.is_empty() || other.is_empty() || range.is_empty() {
            None
        } else {
            Some(range)
        }
    }
}

// Increment a character by a certain number.
pub fn char_add(c: char, i: u32) -> char {
    std::char::from_u32((c as u32) + i).unwrap_or(c)
}

// Allows for different answer types.
#[derive(Debug, PartialEq, Eq)]
pub enum Answer {
    Unsigned(u64),
    Signed(i64),
    String(String),
}
impl From<u64> for Answer {
    fn from(n: u64) -> Self {
        Answer::Unsigned(n)
    }
}
impl From<i64> for Answer {
    fn from(n: i64) -> Self {
        Answer::Signed(n)
    }
}
impl From<String> for Answer {
    fn from(s: String) -> Self {
        Answer::String(s)
    }
}

/// Represents data that can be passed to a solver function.
pub enum SolverInput<'a> {
    Text(&'a str),
    Data(Box<dyn Any>),
}
impl<'a> SolverInput<'a> {
    pub fn expect_input(&self) -> AocResult<&'a str> {
        if let Self::Text(s) = self {
            Ok(s)
        } else {
            Err(AocError::InvalidInput(
                "Expected string input but got something else".into(),
            ))
        }
    }

    pub fn expect_data<T: 'static>(&self) -> AocResult<&T> {
        if let Self::Data(obj) = self {
            obj.downcast_ref::<T>().ok_or(AocError::InvalidInput(
                "Expected data of one type but got a different type".into(),
            ))
        } else {
            Err(AocError::InvalidInput(
                "Expected data input but got something else".into(),
            ))
        }
    }
}
impl<'a> From<&'a str> for SolverInput<'a> {
    fn from(value: &'a str) -> Self {
        Self::Text(value)
    }
}
impl<T: Any> From<Box<T>> for SolverInput<'_> {
    fn from(value: Box<T>) -> Self {
        Self::Data(value)
    }
}

// Represents the solver for both pars of a day's puzzle.
type SolverFunc = fn(&SolverInput) -> AocResult<Answer>;
pub struct Solution {
    pub day: u32,
    pub name: &'static str,
    pub preprocessor: Option<fn(&str) -> AocResult<SolverInput>>,
    pub solvers: &'static [SolverFunc],
}
impl Solution {
    // Constructs the title.
    pub fn title(&self) -> String {
        format!("Day {}: {}", self.day, self.name)
    }

    // Run preprocessor if applicable
    pub fn preprocess<'a>(&self, input: &'a str) -> AocResult<SolverInput<'a>> {
        if let Some(pf) = self.preprocessor {
            pf(input)
        } else {
            Ok(input.into())
        }
    }

    // Reads the input, runs the solvers, and outputs the answer(s).
    pub fn run_and_print(&self, year: u32) -> anyhow::Result<Vec<Answer>> {
        // Read input for the problem
        let input_path = format!("input/{year}/day_{:02}.txt", self.day);
        let input = fs::read_to_string(&input_path)
            .with_context(|| format!("Could not read input file {input_path}"))?;

        // Run solvers
        let data = self.preprocess(&input)?;
        let results = self
            .solvers
            .iter()
            .map(|s| s(&data))
            .collect::<AocResult<Vec<Answer>>>()?;

        println!("{}", format!("Year {} {}", year, self.title()).yellow());
        for (part, result) in ["one", "two"].into_iter().zip(results.iter()) {
            if results.len() > 1 {
                println!("{}", format!("Part {part}:").bold().underline());
            }
            println!(
                "Answer: {}",
                match result {
                    Answer::Unsigned(n) => n.to_string(),
                    Answer::Signed(n) => n.to_string(),
                    Answer::String(s) => s.to_string(),
                }
            );
        }

        Ok(results)
    }
}

// Package of solutions of a year's puzzles.
pub struct YearSolutions {
    pub year: u32,
    pub solutions: &'static [Solution],
}
impl YearSolutions {
    pub fn get_day(&self, day: u32) -> Option<&Solution> {
        self.solutions.iter().find(|s| s.day == day)
    }

    pub fn solution_list(&self) -> String {
        self.solutions
            .iter()
            .map(|solution| solution.title())
            .join("\n")
    }
}

// Convenience trait to convert a vector of numbers into numberic answers.
pub trait AnswerVec {
    fn answer_vec(self) -> Vec<Option<Answer>>;
}
impl AnswerVec for Vec<u64> {
    fn answer_vec(self) -> Vec<Option<Answer>> {
        self.into_iter()
            .map(|n| Some(Answer::Unsigned(n)))
            .collect()
    }
}
impl AnswerVec for Vec<i64> {
    fn answer_vec(self) -> Vec<Option<Answer>> {
        self.into_iter().map(|n| Some(Answer::Signed(n))).collect()
    }
}
impl AnswerVec for Vec<&str> {
    fn answer_vec(self) -> Vec<Option<Answer>> {
        self.into_iter()
            .map(|s| Some(Answer::String(s.into())))
            .collect()
    }
}

// Compares solution results with a vector.
#[macro_export]
macro_rules! solution_results {
    ($input:expr, $exp: expr) => {
        let vans: Vec<Option<Answer>> = $exp;

        let data = SOLUTION.preprocess($input).unwrap();

        for (solver, ans) in SOLUTION.solvers.iter().zip(vans.into_iter()) {
            if let Some(a) = ans {
                assert_eq!(solver(&data).unwrap(), a);
            }
        }
    };
}

// Convenience macro to build the example test for a solution.
// Also creates an ignored test to test the main problem solutions.
#[macro_export]
macro_rules! solution_test {
    ($actual: expr, $($input:expr, $exp: expr), +) => {
        #[test]
        #[ignore]
        fn actual() {
            assert_eq!(
                SOLUTION.run_and_print(super::super::YEAR_SOLUTIONS.year).unwrap(),
                $actual
            );
        }

        #[test]
        fn example() {
            use $crate::solution_results;
            $(
            solution_results!($input, $exp);
            )+
        }
    };
}

// Builds expensive tests that take a while to run.
#[macro_export]
macro_rules! expensive_test {
    ($($input:expr, $exp: expr), +) => {
        #[test]
	    #[cfg(feature = "expensive")]
        fn expensive() {
            use $crate::solution_results;
                $(
            solution_results!($input, $exp);
                )+
        }
    };
}

// Convenience macro to construct the solutions for a year.
#[macro_export]
macro_rules! year_solutions {
    (year = $year: expr, days =  {$($day: ident,)* }) => {
	$(
	    pub mod $day;
	)*

	use aoc::YearSolutions;

	// All of the solutions.
	pub const YEAR_SOLUTIONS: YearSolutions = YearSolutions {
	    year: $year,
	    solutions: &[
		$(
		    $day::SOLUTION,
		)*
	    ],
	};
    }
}
