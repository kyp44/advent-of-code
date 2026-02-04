//! Collection of items related to parsing using [`nom`].
//!
//! Contains some extension traits and useful [`nom`] parsers.

use nom::bytes::complete::tag;
use nom::character::complete::{multispace0, satisfy, space0, space1};
use nom::sequence::{delimited, separated_pair};
use nom::{AsChar, Compare, Input, Parser};
use nom::{Finish, IResult, error::ErrorKind};
use nom::{character::complete::digit1, combinator::map};
use nom_language::error::{VerboseError, VerboseErrorKind};
use num::Unsigned;
use std::fmt;
use std::ops::RangeInclusive;
use std::str::FromStr;

use crate::prelude::{AocError, AocResult};

/// Type of nom input when parsing bits.
pub type BitInput<'a> = (&'a [u8], usize);

/// Custom error type for [`nom`] parsing errors.
///
/// This is needed because the desired nom [`VerboseError`]
/// keeps references to the input string where that could not be parsed.
/// This does not play well with [`anyhow`], which requires that its errors have
/// static lifetime since the error chain is passed out of the main function.
#[derive(Debug, Clone)]
pub struct NomParseError {
    /// The corresponding [`VerboseError`] with an owned string.
    verbose_error: VerboseError<String>,
}
impl PartialEq for NomParseError {
    fn eq(&self, other: &Self) -> bool {
        self.verbose_error.errors == other.verbose_error.errors
    }
}
impl Eq for NomParseError {}
impl nom::error::ParseError<&str> for NomParseError {
    fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
        Self {
            verbose_error: VerboseError::from_error_kind(input.to_string(), kind),
        }
    }

    fn append(input: &str, kind: ErrorKind, other: Self) -> Self {
        Self {
            verbose_error: VerboseError::append(input.to_string(), kind, other.verbose_error),
        }
    }
}
/// A static string when displaying errors innvolving parsing bits.
const BITS_STR: &str = "(bits)";
impl nom::error::ParseError<BitInput<'_>> for NomParseError {
    fn from_error_kind(_input: BitInput, kind: ErrorKind) -> Self {
        Self {
            verbose_error: VerboseError::from_error_kind(BITS_STR.to_string(), kind),
        }
    }

    fn append(_input: BitInput, kind: ErrorKind, other: Self) -> Self {
        Self {
            verbose_error: VerboseError::append(BITS_STR.to_string(), kind, other.verbose_error),
        }
    }
}
impl nom::error::ContextError<&str> for NomParseError {
    fn add_context(_input: &str, _ctx: &'static str, other: Self) -> Self {
        Self {
            verbose_error: VerboseError::add_context(_input.into(), _ctx, other.verbose_error),
        }
    }
}
impl nom::error::ContextError<BitInput<'_>> for NomParseError {
    fn add_context(_input: BitInput<'_>, _ctx: &'static str, other: Self) -> Self {
        Self {
            verbose_error: VerboseError::add_context(BITS_STR.into(), _ctx, other.verbose_error),
        }
    }
}
impl fmt::Display for NomParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.verbose_error, f)
    }
}
impl NomParseError {
    /// Creates a parse error with a context string when parsing bits.
    pub fn nom_err_for_bits(msg: &'static str) -> nom::Err<Self> {
        nom::Err::Failure(NomParseError {
            verbose_error: VerboseError {
                errors: vec![(BITS_STR.to_string(), VerboseErrorKind::Context(msg))],
            },
        })
    }
}
impl std::error::Error for NomParseError {}

/// Type representing the result of a [`nom`] parsing.
pub type NomParseResult<I, U> = IResult<I, U, NomParseError>;

/// Extension trait that simply discards the input portion of a [`nom`]
/// result.
///
/// This should be a part of the nom library in my opinion.
pub trait DiscardInput<U, E> {
    /// Discards the input portion of a [`nom`] result and returns a [`Result`]
    /// without the input such that an [`Ok`] variant will contain only the
    /// parsed value.
    fn discard_input(self) -> Result<U, E>;
}
impl<I, U, E> DiscardInput<U, E> for Result<(I, U), E> {
    fn discard_input(self) -> Result<U, E> {
        self.map(|(_, o)| o)
    }
}

/// Trait for types that can be parsed from text with [`nom`].
pub trait Parsable<'a> {
    /// Needs to parse the text using [`nom`] and return the parsed item.
    fn parser(input: &'a str) -> NomParseResult<&'a str, Self>
    where
        Self: Sized;

    /// Runs the parser and gets the result, stripping out the input from the
    /// nom parser.
    ///
    /// Note that we cannot blanket implement [`FromStr`] for types that
    /// implement this trait because this potentially breaks the orphan
    /// rule. See [here](https://users.rust-lang.org/t/impl-foreign-trait-for-type-bound-by-local-trait/36299).
    fn from_str(input: &'a str) -> Result<Self, NomParseError>
    where
        Self: Sized,
    {
        Self::parser(input).finish().discard_input()
    }

    /// Gathers a [`Vec`] of items from an iterator with each item being a
    /// string from which to parse the item.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # #![feature(assert_matches)]
    /// # use std::assert_matches;
    /// # use aoc::prelude::*;
    /// assert_eq!(
    ///     u8::gather(vec!["43", "22", "5", "8"].into_iter()),
    ///     Ok(vec![43, 22, 5, 8])
    /// );
    /// assert_matches!(
    ///     u8::gather(vec!["43", "22", "5", "text"].into_iter()),
    ///     Err(_)
    /// );
    /// ```
    fn gather(strs: impl Iterator<Item = &'a str>) -> Result<Vec<Self>, NomParseError>
    where
        Self: Sized,
    {
        strs.map(|l| Self::from_str(l)).collect()
    }

    /// Gathers a [`Vec`] of items from a single string in which each item
    /// string is separated by commas.
    ///
    /// No whitespace is allowed between items, but this can be enabled by
    /// wrapping the item parser in the [`trim`] parser.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # #![feature(assert_matches)]
    /// # use std::assert_matches;
    /// # use aoc::prelude::*;
    /// assert_eq!(u8::from_csv("21,27,82,7"), Ok(vec![21, 27, 82, 7]));
    /// assert_matches!(u8::from_csv("21,-56,82,7"), Err(_));
    /// ```
    fn from_csv(input: &'a str) -> Result<Vec<Self>, NomParseError>
    where
        Self: Sized,
    {
        Self::gather(input.split(','))
    }
}

/// [`Parsable`] implementation for simple numbers.
impl<T: Unsigned + FromStr> Parsable<'_> for T {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        map(digit1, |ns: &str| match ns.parse() {
            Ok(v) => v,
            Err(_) => panic!("nom did not parse a numeric value correctly"),
        })
        .parse(input.trim())
    }
}

/// Trims whitespace surrounding a parser.
///
/// This is a [`nom`] combinator.
///
/// # Examples
/// Basic usage:
/// ```
/// # #![feature(assert_matches)]
/// # use std::assert_matches;
/// # use aoc::prelude::*;
/// # use aoc::parse::trim;
/// assert_matches!(
///     nom::character::complete::i32::<_, NomParseError>("   -45   ").discard_input(),
///     Err(_)
/// );
/// assert_eq!(
///     trim(false, nom::character::complete::i32::<_, NomParseError>)
///         .parse("   -45   ")
///         .discard_input(),
///     Ok(-45)
/// );
/// assert_matches!(
///     nom::character::complete::u8::<_, NomParseError>("\n67\n").discard_input(),
///     Err(_)
/// );
/// assert_matches!(
///     trim(false, nom::character::complete::i32::<_, NomParseError>)
///         .parse("\n67\n")
///         .discard_input(),
///     Err(_)
/// );
/// assert_eq!(
///     trim(true, nom::character::complete::i32::<_, NomParseError>)
///         .parse("\n67\n")
///         .discard_input(),
///     Ok(67)
/// );
/// ```
pub fn trim<I, F>(
    include_newlines: bool,
    inner: F,
) -> impl Parser<I, Error = F::Error, Output = F::Output>
where
    I: Input,
    <I as Input>::Item: AsChar + Clone,
    F: Parser<I>,
{
    let space_parser = if include_newlines {
        multispace0
    } else {
        space0
    };

    delimited(space_parser, inner, space_parser)
}

/// Parses only a single alphanumeric character from a string.
///
/// This is a [`nom`] parser that is somehow not included in [`nom`] itself.
///
/// # Examples
/// Basic usage:
/// ```
/// # #![feature(assert_matches)]
/// # use std::assert_matches;
/// # use aoc::prelude::*;
/// # use aoc::parse::single_alphanumeric;
/// assert_eq!(
///     single_alphanumeric::<_, NomParseError>("test").discard_input(),
///     Ok('t')
/// );
/// assert_eq!(
///     single_alphanumeric::<_, NomParseError>("67").discard_input(),
///     Ok('6')
/// );
/// assert_eq!(
///     single_alphanumeric::<_, NomParseError>("TEST").discard_input(),
///     Ok('T')
/// );
/// assert_matches!(
///     single_alphanumeric::<_, NomParseError>("-67").discard_input(),
///     Err(_)
/// );
/// assert_matches!(
///     single_alphanumeric::<_, NomParseError>("&").discard_input(),
///     Err(_)
/// );
/// ```
pub fn single_alphanumeric<I, E>(input: I) -> IResult<I, char, E>
where
    I: Input,
    <I as Input>::Item: AsChar,
    E: nom::error::ParseError<I>,
{
    satisfy(|c| c.is_alphanum())(input)
}

/// Requires whitespace around a parser.
///
/// This is [`nom`] combinator that requires at least some whitespace before
/// and after another parser in order to succeed. The whitespace does not
/// include newlines.
///
/// # Examples
/// Basic usage:
/// ```
/// # #![feature(assert_matches)]
/// # use std::assert_matches;
/// # use aoc::prelude::*;
/// # use aoc::parse::separated;
/// assert_matches!(
///     separated(nom::character::complete::i32::<_, NomParseError>)
///         .parse("64")
///         .discard_input(),
///     Err(_)
/// );
/// assert_eq!(
///     separated(nom::character::complete::i32::<_, NomParseError>)
///         .parse(" 64 ")
///         .discard_input(),
///     Ok(64)
/// );
/// assert_eq!(
///     separated(nom::character::complete::i32::<_, NomParseError>)
///         .parse("    64  ")
///         .discard_input(),
///     Ok(64)
/// );
/// assert_matches!(
///     separated(nom::character::complete::i32::<_, NomParseError>)
///         .parse("\n64\n")
///         .discard_input(),
///     Err(_)
/// );
/// assert_matches!(
///     separated(nom::character::complete::i32::<_, NomParseError>)
///         .parse("\n  64  \n")
///         .discard_input(),
///     Err(_)
/// );
/// assert_matches!(
///     separated(nom::character::complete::i32::<_, NomParseError>)
///         .parse("   \n64\n  ")
///         .discard_input(),
///     Err(_)
/// );
/// ```
pub fn separated<I, F>(inner: F) -> impl Parser<I, Error = F::Error, Output = F::Output>
where
    I: Input,
    <I as Input>::Item: AsChar + Clone,
    F: Parser<I>,
{
    delimited(space1, inner, space1)
}

/// Parses a single decimal digit.
///
/// This is a [`nom`] parser.
///
/// # Examples
/// Basic usage:
/// ```
/// # #![feature(assert_matches)]
/// # use std::assert_matches;
/// # use aoc::prelude::*;
/// # use aoc::parse::single_digit;
/// assert_eq!(
///     single_digit::<_, NomParseError>("76").discard_input(),
///     Ok(7)
/// );
/// assert_eq!(
///     single_digit::<_, NomParseError>("0text").discard_input(),
///     Ok(0)
/// );
/// assert_matches!(
///     single_digit::<_, NomParseError>("text").discard_input(),
///     Err(_)
/// );
/// assert_matches!(
///     single_digit::<_, NomParseError>("-9").discard_input(),
///     Err(_)
/// );
/// ```
pub fn single_digit<I, E>(input: I) -> IResult<I, u8, E>
where
    I: Input,
    <I as Input>::Item: AsChar + Copy,
    E: nom::error::ParseError<I>,
{
    match input
        .iter_elements()
        .next()
        .map(|c| (c, c.as_char().to_digit(10)))
    {
        Some((c, Some(d))) => Ok((input.take_from(c.len()), d.try_into().unwrap())),
        _ => Err(nom::Err::Error(E::from_error_kind(
            input,
            ErrorKind::NoneOf,
        ))),
    }
}

/// Parses a label followed by another parser with potential whitespace in
/// between.
///
/// This is a [`nom`] parser that will also consume any whitespace (including
/// newlines) after the `inner` parser.
///
/// # Examples
/// Basic usage:
/// ```
/// # #![feature(assert_matches)]
/// # use std::assert_matches;
/// # use aoc::prelude::*;
/// # use aoc::parse::field_line_parser;
/// assert_eq!(
///     field_line_parser("name:", nom::character::complete::u8::<_, NomParseError>)
///         .parse("name:        47")
///         .discard_input(),
///     Ok(47)
/// );
/// assert_eq!(
///     field_line_parser(
///         "job =",
///         nom::character::complete::alpha1::<_, NomParseError>
///     )
///     .parse("job =electrician")
///     .discard_input(),
///     Ok("electrician")
/// );
/// assert_matches!(
///     field_line_parser(
///         "class:",
///         nom::character::complete::alpha1::<_, NomParseError>
///     )
///     .parse("class = mage"),
///     Err(_)
/// );
/// ```
pub fn field_line_parser<I, F>(
    label: &'static str,
    inner: F,
) -> impl Parser<I, Error = F::Error, Output = F::Output>
where
    I: Input + Compare<&'static str>,
    <I as Input>::Item: AsChar,
    F: Parser<I>,
{
    delimited(tag(label), trim(false, inner), multispace0)
}

/// Parses an inclusive range.
///
/// This is a [`nom`] parser parses two numbers with a dash in between as
/// an inclusive range.
/// The numeric type is determined from the `inner` parser output type.
/// Whitespace is allowed between the dash and the numbers, but not newlines.
///
/// # Examples
/// Basic usage:
/// ```
/// # #![feature(assert_matches)]
/// # use std::assert_matches;
/// # use aoc::prelude::*;
/// # use aoc::parse::inclusive_range;
/// assert_eq!(
///     inclusive_range(nom::character::complete::u8::<_, NomParseError>)
///         .parse("4-13")
///         .discard_input(),
///     Ok(4..=13)
/// );
/// assert_eq!(
///     inclusive_range(nom::character::complete::i32::<_, NomParseError>)
///         .parse("-89765 - -1234")
///         .discard_input(),
///     Ok(-89765..=-1234)
/// );
/// assert_matches!(
///     inclusive_range(nom::character::complete::u16::<_, NomParseError>).parse("1-xyz"),
///     Err(_)
/// );
/// ```
pub fn inclusive_range<'a, I, F>(
    inner: F,
) -> impl Parser<I, Error = F::Error, Output = RangeInclusive<F::Output>>
where
    I: Input + Compare<&'a str>,
    <I as Input>::Item: AsChar,
    F: Parser<I> + Copy,
{
    map(
        separated_pair(inner, delimited(space0, tag("-"), space0), inner),
        |(a, b)| a..=b,
    )
}

/// Extension trait to break a string into some number of section substrings.
pub trait Sections {
    /// Breaks the string into `num` sections.
    ///
    /// Each section is separated by a double newline. This will fail if
    /// the input string does not contain exactly the correct number of
    /// sections.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # #![feature(assert_matches)]
    /// # use std::assert_matches;
    /// # use aoc::prelude::*;
    /// assert_eq!(
    ///     "section 1\n\nsection 2\n\nsection 3".sections(3),
    ///     Ok(vec!["section 1", "section 2", "section 3"])
    /// );
    /// assert_eq!(
    ///     "section\n1\n\nsection\n2\n\nsection\n3".sections(3),
    ///     Ok(vec!["section\n1", "section\n2", "section\n3"])
    /// );
    /// assert_matches!("section 1\n\nsection 2".sections(3), Err(_));
    /// assert_matches!(
    ///     "section 1\n\nsection 2\n\nsection 3\n\nsection 4".sections(3),
    ///     Err(_)
    /// );
    /// assert_matches!("section 1\nsection 2\nsection 3".sections(3), Err(_));
    /// ```
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
