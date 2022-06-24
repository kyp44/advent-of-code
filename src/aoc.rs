use anyhow::Context;
use itertools::Itertools;
use num::Integer;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::RangeInclusive;
use std::{fmt, fs};

use self::parse::NomParseError;

mod evolver;
mod grid;
mod iter;
pub mod parse;

/// Prelude
pub mod prelude {
    pub use super::{
        char_add, evolver::Evolver, grid::CharGrid, grid::Grid, grid::GridPoint, grid::GridSize,
        grid::GridSizeExt, grid::PointTryInto, iter::FilterCount, iter::HasNoneIter,
        iter::HasRange, iter::IndividualReplacements, iter::SplitRuns, parse::BitInput,
        parse::DiscardInput, parse::NomParseError, parse::NomParseResult, parse::Parseable,
        parse::Sections, Answer, AnswerVec, AocError, AocResult, RangeExt, Solution, SolverData,
        YearSolutions,
    };
    pub use aoc_derive::CharGridDebug;
}

/// Custom error type for AoC problem functions.
#[derive(Debug, Clone)]
pub enum AocError {
    NoYear(u32),
    NoDay(u32),
    DayRange(u32, RangeInclusive<u32>),
    NomParse(NomParseError),
    InvalidInput(Cow<'static, str>),
    Process(Cow<'static, str>),
}
impl Display for AocError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AocError::NoYear(y) => write!(f, "Year {} is not yet solved", y),
            AocError::NoDay(d) => write!(f, "Day {} is not yet solved", d),
            AocError::DayRange(d, r) => write!(
                f,
                "Day {} is not in the range of {}-{}",
                d,
                r.start(),
                r.end()
            ),
            AocError::NomParse(e) => write!(f, "{}", e),
            AocError::InvalidInput(s) => write!(f, "Invalid input: {}", s),
            AocError::Process(s) => write!(f, "Error while processing: {}", s),
        }
    }
}
impl Error for AocError {}
impl From<NomParseError> for AocError {
    fn from(e: NomParseError) -> Self {
        AocError::NomParse(e)
    }
}
pub type AocResult<T> = Result<T, AocError>;

/// Extension trait for ranges.
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
        if self.is_empty() || other.is_empty() {
            None
        } else if self.start() <= other.start() {
            if self.end() < other.start() {
                None
            } else {
                Some(range)
            }
        } else {
            if other.end() < other.start() {
                None
            } else {
                Some(range)
            }
        }
    }
}

/// Increment a character by a certain number.
pub fn char_add(c: char, i: u32) -> char {
    std::char::from_u32((c as u32) + i).unwrap_or(c)
}

/// Allows for different answer types.
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

use super::aoc_2021::day_19::BeaconScannerData;

//// Represents data that can be passed to or from solver.
pub enum SolverData<'a> {
    Input(&'a str),
    Year2021Day19(BeaconScannerData),
}
impl<'a> SolverData<'a> {
    pub fn expect_input(&self) -> AocResult<&'a str> {
        if let Self::Input(s) = self {
            Ok(s)
        } else {
            Err(AocError::InvalidInput(
                "Expected string input but got data".into(),
            ))
        }
    }
}

/// To easily extract data from SolverData and create an error if not there
#[macro_export]
macro_rules! expect_data {
    ($var:ident, $input:expr) => {
        if let SolverData::$var(it) = $input {
            Ok(it)
        } else {
            Err(AocError::InvalidInput(
                format!("Expected {} data but did not get it", stringify!($var)).into(),
            ))
        }
    };
}

/// Represents the solver for a day's puzzle.
type SolverFunc = fn(&SolverData) -> AocResult<Answer>;
pub struct Solution {
    pub day: u32,
    pub name: &'static str,
    pub preprocessor: Option<fn(&str) -> AocResult<SolverData>>,
    pub solvers: &'static [SolverFunc],
}
impl Solution {
    /// Constructs the title.
    pub fn title(&self) -> String {
        format!("Day {}: {}", self.day, self.name)
    }

    /// Run preprocessor if applicable
    pub fn preprocess<'a>(&self, input: &'a str) -> AocResult<SolverData<'a>> {
        if let Some(pf) = self.preprocessor {
            pf(input)
        } else {
            Ok(SolverData::Input(input))
        }
    }

    /// Reads the input, runs the solvers, and outputs the answer(s).
    pub fn run_and_print(&self, year: u32) -> anyhow::Result<Vec<Answer>> {
        // Read input for the problem
        let input_path = format!("input/{}/day_{:02}.txt", year, self.day);
        let input = fs::read_to_string(&input_path)
            .with_context(|| format!("Could not read input file {}", input_path))?;

        // Run solvers
        let data = self.preprocess(&input)?;
        let results = self
            .solvers
            .iter()
            .map(|s| s(&data))
            .collect::<AocResult<Vec<Answer>>>()?;

        println!("Year {} {}", year, self.title());
        for (pc, result) in ('a'..='z').zip(results.iter()) {
            if results.len() > 1 {
                println!("Part {})", pc);
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

/// Package of solutions of a year's puzzles.
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

/// Convenience trait to convert a vector of numbers into numberic answers.
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

/// Compares solution results with a vector.
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

/// Convenience macro to build the example test for a solution.
/// Also creates an ignored test to test the main problem solutions.
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

/// Builds expensive tests that take a while to run.
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

/// Convenience macro to construct the solutions for a year.
#[macro_export]
macro_rules! year_solutions {
    (year = $year: expr, days =  {$($day: ident,)* }) => {
	$(
	    pub mod $day;
	)*

	use super::aoc::YearSolutions;

	/// All of the solutions.
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
