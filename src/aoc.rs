use anyhow::Context;
use itertools::Itertools;
use nom::character::complete::space0;
use nom::sequence::delimited;
use nom::{character::complete::digit1, combinator::map};
use nom::{error::ErrorKind, error::VerboseError, Finish, IResult};
use num::Unsigned;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::{fmt, fs};

/// Prelude
pub mod prelude {
    pub use super::{
        Answer, AnswerVec, AocError, AocResult, DiscardInput, FilterCount, NomParseError,
        NomParseResult, Parseable, Sections, Solution, YearSolutions,
    };
}

/// Custom error type for AoC problem functions.
#[derive(Debug, Clone)]
pub enum AocError {
    NoYear(u32),
    NoDay(u32),
    NomParse(NomParseError),
    InvalidInput(String),
    Process(String),
}
impl Display for AocError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AocError::NoYear(y) => write!(f, "Year {} is not yet solved", y),
            AocError::NoDay(d) => write!(f, "Day {} is not yet solved", d),
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

/// This custom parse error type is needed because the desired Nom VerboseError
/// keeps references to the input string where that could not be parsed.
/// This does not play well with anyhow, which requires that its errors have
/// static lifetime since the error chain is passed out of main().
#[derive(Debug, Clone)]
pub struct NomParseError {
    verbose_error: VerboseError<String>,
}
impl nom::error::ParseError<&str> for NomParseError {
    fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
        NomParseError {
            verbose_error: VerboseError::from_error_kind(input.to_string(), kind),
        }
    }

    fn append(input: &str, kind: ErrorKind, other: Self) -> Self {
        NomParseError {
            verbose_error: VerboseError::append(input.to_string(), kind, other.verbose_error),
        }
    }
}
impl nom::error::ContextError<&str> for NomParseError {}
impl Display for NomParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.verbose_error, f)
    }
}

/// Type containing the result of a nom parsing
pub type NomParseResult<'a, U> = IResult<&'a str, U, NomParseError>;

/// Trait for types to be parsable with Nom.
/// Note that we cannot simply implement FromStr for types that implement this trait
/// because this breaks the potential foreign trait on a foreign type rules.
/// See here: https://users.rust-lang.org/t/impl-foreign-trait-for-type-bound-by-local-trait/36299
pub trait Parseable<'a> {
    /// Parser function for nom
    fn parser(input: &'a str) -> NomParseResult<Self>
    where
        Self: Sized;

    /// Runs the parser and gets the result, stripping out the input from the nom parser
    fn from_str(input: &'a str) -> Result<Self, NomParseError>
    where
        Self: Sized,
    {
        Self::parser(input).finish().map(|t| t.1)
    }

    /// Gathers a vector of items from an iterator with each item being a string to parse
    fn gather<I>(strs: I) -> Result<Vec<Self>, NomParseError>
    where
        Self: Sized,
        I: Iterator<Item = &'a str>,
    {
        strs.map(|l| Self::from_str(l))
            .collect::<Result<Vec<Self>, NomParseError>>()
    }

    fn from_csv(input: &'a str) -> Result<Vec<Self>, NomParseError>
    where
        Self: Sized,
    {
        input.split(',').map(|s| Self::from_str(s)).collect()
    }
}

/// Parseable for simple numbers
impl<T: Unsigned + FromStr> Parseable<'_> for T {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(digit1, |ns: &str| match ns.parse() {
            Ok(v) => v,
            Err(_) => panic!("nom did not parse a numeric value correctly"),
        })(input.trim())
    }
}

/// A nom combinator that trims whitespace surrounding a parser
pub fn trim<'a, F: 'a, O, E: nom::error::ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(space0, inner, space0)
}

/// A nom parser that takes a single decimal digit
fn single_digit<'a, E>(input: &'a str) -> IResult<&str, u8, E>
where
    E: nom::error::ParseError<&'a str>,
{
    let ds = &input[..1];
    todo!()
}

/// This should be a part of the nom library in my opinion
pub trait DiscardInput<U, E> {
    fn discard_input(self) -> Result<U, E>;
}
impl<I, U, E> DiscardInput<U, E> for Result<(I, U), E> {
    fn discard_input(self) -> Result<U, E> {
        self.map(|(_, o)| o)
    }
}

pub trait Sections {
    fn sections(&self, num: usize) -> AocResult<Vec<&str>>;
}
impl Sections for str {
    fn sections(&self, num: usize) -> AocResult<Vec<&str>> {
        let secs: Vec<&str> = self.split("\n\n").collect();
        if secs.len() == num {
            Ok(secs)
        } else {
            Err(AocError::InvalidInput(format!(
                "Expected {} sections from the input, found {}",
                num,
                secs.len()
            )))
        }
    }
}

/// Convenience function to count from a filtered Iterator
pub trait FilterCount<T, O> {
    fn filter_count<F: Fn(&T) -> bool>(self, f: F) -> O;
}
impl<T, I, O: TryFrom<usize>> FilterCount<T, O> for I
where
    I: Iterator<Item = T>,
    <O as TryFrom<usize>>::Error: Debug,
{
    fn filter_count<F: Fn(&T) -> bool>(self, f: F) -> O {
        self.filter(f).count().try_into().unwrap()
    }
}

/// Allows for different answer types
#[derive(Debug, PartialEq, Eq)]
pub enum Answer {
    Number(u64),
    String(String),
}
impl From<u64> for Answer {
    fn from(n: u64) -> Self {
        Answer::Number(n)
    }
}
impl From<String> for Answer {
    fn from(s: String) -> Self {
        Answer::String(s)
    }
}

/// Represents the solver for a day's puzzle.
pub struct Solution {
    pub day: u32,
    pub name: &'static str,
    pub solvers: &'static [fn(&str) -> AocResult<Answer>],
}
impl Solution {
    /// Constructs the title
    pub fn title(&self) -> String {
        format!("Day {}: {}", self.day, self.name)
    }

    /// Reads the input, runs the solvers, and outputs the answer(s).
    pub fn run(&self, year: u32) -> anyhow::Result<Vec<Answer>> {
        // Read input for the problem
        let input_path = format!("input/{}/day_{:02}.txt", year, self.day);
        let input = fs::read_to_string(&input_path)
            .with_context(|| format!("Could not read input file {}", input_path))?;

        let results = self
            .solvers
            .iter()
            .map(|s| s(&input).with_context(|| "Problem when running the solution"))
            .collect::<anyhow::Result<Vec<Answer>>>()?;

        println!("Year {} {}", year, self.title());
        for (pc, result) in ('a'..'z').zip(results.iter()) {
            if results.len() > 1 {
                println!("Part {})", pc);
            }
            println!(
                "Answer: {}",
                match result {
                    Answer::Number(n) => n.to_string(),
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

/// Convenience trait to convert a vector of numbers into numberic answers
pub trait AnswerVec {
    fn answer_vec(self) -> Vec<Option<Answer>>;
}
impl AnswerVec for Vec<u64> {
    fn answer_vec(self) -> Vec<Option<Answer>> {
        self.into_iter().map(|n| Some(Answer::Number(n))).collect()
    }
}

/// Compares solution results with a vector
#[macro_export]
macro_rules! solution_results {
    ($input:literal, $exp: expr) => {
        let input = $input;
        let vans: Vec<Option<Answer>> = $exp;

        for (solver, ans) in SOLUTION.solvers.iter().zip(vans) {
            if let Some(v) = ans {
                assert_eq!(solver(&input).unwrap(), v);
            }
        }
    };
}

/// Convenience macro to build the example test for a solution.
/// Also creates an ignored test to test the main problem solutions.
#[macro_export]
macro_rules! solution_test {
    ($actual: expr, $($input:literal, $exp: expr), +) => {
        #[test]
        #[ignore]
        fn actual() {
            assert_eq!(
                SOLUTION.run(super::super::YEAR_SOLUTIONS.year).unwrap(),
                $actual
            );
        }

        #[test]
        fn example() {
	    use crate::solution_results;
	    $(
		solution_results!($input, $exp);
	    )+
        }
    };
}

/// Builds expensive tests that take a while to run.
#[macro_export]
macro_rules! expensive_test {
    ($($input:literal, $exp: expr), +) => {
        #[test]
	#[cfg(feature = "expensive")]
        fn expensive() {
	    use crate::solution_results;
            $(
		solution_results!($input, $exp);
            )+
        }
    };
}

/// Convenience macro to construct the solutions for a year
#[macro_export]
macro_rules! year_solutions {
    (year = $year: expr, days =  {$($day: ident,)* }) => {
	$(
	    mod $day;
	)*

	use super::aoc::YearSolutions;

	/// All of the solutions
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
