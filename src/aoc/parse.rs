use super::{AocError, AocResult};
use nom::bytes::complete::tag;
use nom::character::complete::{multispace0, space0, space1};
use nom::sequence::delimited;
use nom::{character::complete::digit1, combinator::map};
use nom::{error::ErrorKind, error::VerboseError, Finish, IResult};
use nom::{AsChar, InputIter, InputTakeAtPosition, Slice};
use num::Unsigned;
use std::fmt;
use std::ops::RangeFrom;
use std::str::FromStr;

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
impl fmt::Display for NomParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.verbose_error, f)
    }
}

/// Type containing the result of a nom parsing.
pub type NomParseResult<'a, U> = IResult<&'a str, U, NomParseError>;

/// This should be a part of the nom library in my opinion.
pub trait DiscardInput<U, E> {
    fn discard_input(self) -> Result<U, E>;
}
impl<I, U, E> DiscardInput<U, E> for Result<(I, U), E> {
    fn discard_input(self) -> Result<U, E> {
        self.map(|(_, o)| o)
    }
}

/// Trait for types to be parsable with Nom.
/// Note that we cannot simply implement FromStr for types that implement this trait
/// because this breaks the potential foreign trait on a foreign type rules.
/// See here: https://users.rust-lang.org/t/impl-foreign-trait-for-type-bound-by-local-trait/36299
pub trait Parseable<'a> {
    /// Parser function for nom.
    fn parser(input: &'a str) -> NomParseResult<Self>
    where
        Self: Sized;

    /// Runs the parser and gets the result, stripping out the input from the nom parser.
    fn from_str(input: &'a str) -> Result<Self, NomParseError>
    where
        Self: Sized,
    {
        Self::parser(input).finish().map(|t| t.1)
    }

    /// Gathers a vector of items from an iterator with each item being a string to parse.
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

/// Parseable for simple numbers.
impl<T: Unsigned + FromStr> Parseable<'_> for T {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(digit1, |ns: &str| match ns.parse() {
            Ok(v) => v,
            Err(_) => panic!("nom did not parse a numeric value correctly"),
        })(input.trim())
    }
}

/// A nom combinator that trims whitespace surrounding a parser.
pub fn trim<I, F, O, E>(inner: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: InputTakeAtPosition,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    F: FnMut(I) -> IResult<I, O, E>,
    E: nom::error::ParseError<I>,
{
    delimited(space0, inner, space0)
}

/// A nom combinator that requires some whitespace around a parser.
pub fn separated<I, F, O, E>(inner: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: InputTakeAtPosition,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    F: FnMut(I) -> IResult<I, O, E>,
    E: nom::error::ParseError<I>,
{
    delimited(space1, inner, space1)
}

/// A nom parser that takes a single decimal digit.
pub fn single_digit<I, E>(input: I) -> IResult<I, u32, E>
where
    I: Slice<RangeFrom<usize>> + InputIter,
    <I as InputIter>::Item: AsChar + Copy,
    E: nom::error::ParseError<I>,
{
    match input
        .iter_elements()
        .next()
        .map(|c| (c, c.as_char().to_digit(10)))
    {
        Some((c, Some(d))) => Ok((input.slice(c.len()..), d)),
        _ => Err(nom::Err::Error(E::from_error_kind(
            input,
            ErrorKind::NoneOf,
        ))),
    }
}

/// Parses a label followed by another parser with potential whitespace in between
/// on a single line.
pub fn field_line_parser<'a, F, O, E>(
    label: &'static str,
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    E: nom::error::ParseError<&'a str>,
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(tag(label), trim(inner), multispace0)
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
            Err(AocError::InvalidInput(
                format!(
                    "Expected {} sections from the input, found {}",
                    num,
                    secs.len()
                )
                .into(),
            ))
        }
    }
}
