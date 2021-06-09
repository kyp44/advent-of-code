use std::error::Error;
use std::fmt;
use nom::{IResult, error::VerboseError};

#[derive(Debug, Clone)]
pub enum AocError<'a> {
    Parse(ParseError<&'a str>),
}

impl fmt::Display for AocError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AocError::Parse(e) => e.fmt(f)
        }
    }
}

impl Error for AocError<'_> {
    
}

/// Parse error type
pub type ParseError<T> = VerboseError<T>;

/// Type containing the result of a nom parser
pub type ParseResult<T, U> = IResult<T, U, ParseError<T>>;

