use std::error::Error;
use std::{fmt, fs};
use nom::{IResult, Err as NomErr, error::VerboseError, error::ErrorKind};
use anyhow::{Context, Result};

/// Custom error type for AoC problem functions.
#[derive(Debug, Clone)]
pub enum AocError {
    NoYear(u32),
    NoDay(u32),
    NomParse(NomErr<ParseError>),
    InvalidInput(String),
    Process(String),
}
impl fmt::Display for AocError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AocError::NoYear(y) => write!(f, "Year {} is not yet solved", y),
            AocError::NoDay(d) => write!(f, "Day {} is not yet solved", d),
            AocError::NomParse(ne) => {
                write!(f, "Parsing problem: ")?;
                match ne {
                    NomErr::Incomplete(_) => write!(f, "Incomplete parse"),
                    NomErr::Error(e) | NomErr::Failure(e) => write!(f, "{}", e),
                }
            },
            AocError::InvalidInput(s) => write!(f, "Invalid input: {}", s),
            AocError::Process(s) => write!(f, "Error while processing: {}", s),

        }
    }
}
impl Error for AocError {}
impl From<NomErr<ParseError>> for AocError {
    fn from(e: NomErr<ParseError>) -> Self {
        AocError::NomParse(e)
    }
}

/// This custom parse error type is needed because the desired Nom VerboseError
/// keeps references to the input string where that could not be parsed.
/// This does not play well with anyhow, which requires that its errors have
/// static lifetime since the error chain is passed out of main().
#[derive(Debug, Clone)]
pub struct ParseError {
    verbose_error: VerboseError<String>
}
impl nom::error::ParseError<&str> for ParseError {
    fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
        ParseError {
            verbose_error: VerboseError::from_error_kind(input.to_string(), kind)
        }
    }

    fn append(input: &str, kind: ErrorKind, other: Self) -> Self {
        ParseError {
            verbose_error: VerboseError::append(input.to_string(), kind, other.verbose_error)
        }
    }
}
impl nom::error::ContextError<&str> for ParseError {}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.verbose_error.fmt(f)
    }
}

/// Trait for types to be parsable with Nom
pub trait Parseable {
    /// Parser function for nom 
    fn parse(input: &str) -> ParseResult<Self> where Self: Sized;

    /// Runs the parser and gets the result, stripping out the input from the nom parser
    fn from_str(input: &str) -> Result<Self, NomErr<ParseError>> where Self: Sized {
        Self::parse(input).map(|t| t.1)
    }

    /// Gathers a vector of items from an iterator with each item being a string to parse
    fn gather<'a, I>(strs: I) -> Result<Vec<Self>, NomErr<ParseError>>
    where
        Self: Sized,
        I: Iterator<Item = &'a str>,
    {
        strs.map(|l| Self::from_str(l))
            .collect::<Result<Vec<Self>, NomErr<ParseError>>>()
    }
}

/// Type containing the result of a nom parsing
pub type ParseResult<'a, U> = IResult<&'a str, U, ParseError>;

/// Represents the solver for a day's puzzle.
pub struct Solution {
    pub day: u32,
    pub name: &'static str,
    pub solver: fn(&str) -> Result<Vec<u32>, AocError>,
}
impl Solution {
    /// Reads the input, runs the solver, and outputs the answer(s).
    pub fn run(&self, year: u32) -> Result<()> {
        // Read input for the problem
        let input_path = format!("input/{}/day_{:02}.txt", year, self.day);
        let input = fs::read_to_string(&input_path)
            .with_context(|| format!("Could not read input file {}", input_path))?;

        let results = (self.solver)(&input).with_context(|| "Problem when running the solution")?;

        println!("{} Day {}: {}", year, self.day, self.name);
        for (pc, result) in ('a'..'z').zip(results.iter()) {
            if results.len() > 1 {
                println!("Part {})", pc);
            }
            println!("Answer: {}", result);
        }

        Ok(())
    }

}

/// Package of solutions of a year's puzzles.
pub struct YearSolutions {
    pub year: u32,
    pub solutions: Vec<Solution>,
}
impl YearSolutions {
    pub fn get_day(&self, day: u32) -> Option<&Solution> {
        self.solutions.iter()
            .find(|s| s.day == day)
    }
}

/// Convenience macro to build the example test for a solution.
#[macro_export]
macro_rules! solution_test {
    ($test: ident, $sol: ident, $in: literal, $cor: expr) => {
        #[test]
        fn $test() {
            let input = $in;
            
            match ($sol.solver)(input) {
                Ok(v) => assert_eq!(v, $cor),
                Err(e) => panic!("{}", e),
            }
        }
    }
}
