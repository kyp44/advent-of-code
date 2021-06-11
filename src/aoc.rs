use std::error::Error;
use std::fmt;
use nom::{IResult, error::VerboseError, error::ErrorKind};

#[derive(Debug, Clone)]
pub enum AocError {
    NoYear(u32),
    NoDay(u32),
    Parse(nom::Err<ParseError>),
}

impl fmt::Display for AocError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AocError::Parse(e) => e.fmt(f),
            AocError::NoYear(y) => write!(f, "Year {} is not yet implemented", y),
            AocError::NoDay(d) => write!(f, "Day {} is not yet implemented", d),
        }
    }
}

impl Error for AocError {}

impl From<nom::Err<ParseError>> for AocError {
    fn from(e: nom::Err<ParseError>) -> Self {
        AocError::Parse(e)
    }
}

/// TODO describe why this is needed.
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

/// Type containing the result of a nom parsing
pub type ParseResult<'a, U> = IResult<&'a str, U, ParseError>;
