use anyhow::Context;
use nom::{character::complete::digit1, combinator::map};
use nom::{error::ErrorKind, error::VerboseError, Finish, IResult};
use num::Unsigned;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::{fmt, fs};

/// Custom error type for AoC problem functions.
#[derive(Debug, Clone)]
pub enum AocError {
    NoYear(u32),
    NoDay(u32),
    NomParse(ParseError),
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
impl From<ParseError> for AocError {
    fn from(e: ParseError) -> Self {
        AocError::NomParse(e)
    }
}
pub type AocResult<T> = Result<T, AocError>;

/// This custom parse error type is needed because the desired Nom VerboseError
/// keeps references to the input string where that could not be parsed.
/// This does not play well with anyhow, which requires that its errors have
/// static lifetime since the error chain is passed out of main().
#[derive(Debug, Clone)]
pub struct ParseError {
    verbose_error: VerboseError<String>,
}
impl nom::error::ParseError<&str> for ParseError {
    fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
        ParseError {
            verbose_error: VerboseError::from_error_kind(input.to_string(), kind),
        }
    }

    fn append(input: &str, kind: ErrorKind, other: Self) -> Self {
        ParseError {
            verbose_error: VerboseError::append(input.to_string(), kind, other.verbose_error),
        }
    }
}
impl nom::error::ContextError<&str> for ParseError {}
impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.verbose_error, f)
    }
}

/// Trait for types to be parsable with Nom.
/// Note that we cannot simply implement FromStr for types that implement this trait
/// because this breaks the potential foreign trait on a foreign type rules.
/// See here: https://users.rust-lang.org/t/impl-foreign-trait-for-type-bound-by-local-trait/36299
pub trait Parseable {
    /// Parser function for nom
    fn parse(input: &str) -> ParseResult<Self>
    where
        Self: Sized;

    /// Runs the parser and gets the result, stripping out the input from the nom parser
    fn from_str(input: &str) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        Self::parse(input).finish().map(|t| t.1)
    }

    /// Gathers a vector of items from an iterator with each item being a string to parse
    fn gather<'a, I>(strs: I) -> Result<Vec<Self>, ParseError>
    where
        Self: Sized,
        I: Iterator<Item = &'a str>,
    {
        strs.map(|l| Self::from_str(l))
            .collect::<Result<Vec<Self>, ParseError>>()
    }

    fn from_csv(input: &str) -> Result<Vec<Self>, ParseError>
    where
        Self: Sized,
    {
        input.split(",").map(|s| Self::from_str(s)).collect()
    }
}

/// Parseable for simple numbers
impl<T: Unsigned + FromStr> Parseable for T {
    fn parse(input: &str) -> ParseResult<Self> {
        map(digit1, |ns: &str| match ns.parse() {
            Ok(v) => v,
            Err(_) => panic!("nom did not parse a numeric value correctly"),
        })(input.trim())
    }
}

/// Type containing the result of a nom parsing
pub type ParseResult<'a, U> = IResult<&'a str, U, ParseError>;

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

/// Represents the solver for a day's puzzle.
pub struct Solution {
    pub day: u32,
    pub name: &'static str,
    pub solvers: &'static [fn(&str) -> AocResult<u64>],
}
impl Solution {
    /// Constructs the title
    pub fn title(&self, year: u32) -> String {
        format!("{} Day {}: {}", year, self.day, self.name)
    }

    /// Reads the input, runs the solvers, and outputs the answer(s).
    pub fn run(&self, year: u32) -> anyhow::Result<Vec<u64>> {
        // Read input for the problem
        let input_path = format!("input/{}/day_{:02}.txt", year, self.day);
        let input = fs::read_to_string(&input_path)
            .with_context(|| format!("Could not read input file {}", input_path))?;

        let results = self
            .solvers
            .iter()
            .map(|s| s(&input).with_context(|| "Problem when running the solution"))
            .collect::<anyhow::Result<Vec<u64>>>()?;

        println!("{}", self.title(year));
        for (pc, result) in ('a'..'z').zip(results.iter()) {
            if results.len() > 1 {
                println!("Part {})", pc);
            }
            println!("Answer: {}", result);
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

    pub fn print_solution_list(&self) {
        for solution in self.solutions.iter() {
            println!("{}", solution.title(self.year));
        }
    }
}

/// Compares solution results with a vector
#[macro_export]
macro_rules! solution_results {
    ($input:literal, $exp: expr) => {
        let input = $input;

        for (solver, vopt) in SOLUTION.solvers.iter().zip($exp) {
            if let Some(v) = vopt {
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
