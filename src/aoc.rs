use std::error::Error;
use std::fmt;
use nom::{IResult, Err as NomErr, error::VerboseError, error::ErrorKind};

/// Custom error type for AoC problem functions.
#[derive(Debug, Clone)]
pub enum AocError {
    NoYear(u32),
    NoDay(u32),
    Parse(NomErr<ParseError>),
    Process(String),
}
impl fmt::Display for AocError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AocError::NoYear(y) => write!(f, "Year {} is not yet implemented", y),
            AocError::NoDay(d) => write!(f, "Day {} is not yet implemented", d),
            AocError::Parse(ne) => {
                write!(f, "Parsing problem: ")?;
                match ne {
                    NomErr::Incomplete(_) => write!(f, "Incomplete parse"),
                    NomErr::Error(e) | NomErr::Failure(e) => write!(f, "{}", e),
                }
            },
            AocError::Process(s) => write!(f, "Error while processing: {}", s),

        }
    }
}
impl Error for AocError {}
impl From<NomErr<ParseError>> for AocError {
    fn from(e: NomErr<ParseError>) -> Self {
        AocError::Parse(e)
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

    /// Gathers a vector of items from an iterator with item being a string to parse
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

/// Tests a result to panic with the error if there is an issue
#[cfg(test)]
pub fn test_result(result: Result<Vec<u32>, super::AocError>, correct: Vec<u32>) {
    match result {
        Ok(v) => assert_eq!(v, correct),
        Err(e) => panic!("{}", e),
    }
}
