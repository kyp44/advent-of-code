use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, digit1},
    combinator::{map, value},
    sequence::{preceded, separated_pair},
    IResult,
};

use crate::aoc::{prelude::*, separator};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![],
    "123 -> x
456 -> y
x AND y -> d
x OR y -> e
x LSHIFT 2 -> f
y RSHIFT 2 -> g
NOT x -> h
NOT y -> i
NOT f -> a",
    vec![65043u64].answer_vec()
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Input<'a> {
    Value(u16),
    Wire(&'a str),
}
impl<'a> Parseable<'a> for Input<'a> {
    fn parser(input: &'a str) -> NomParseResult<Self> {
        alt((
            map(digit1, |ds: &str| Input::Value(ds.parse().unwrap())),
            map(alpha1, |s| Input::Wire(s)),
        ))(input)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Unary<'a> {
    input: Input<'a>,
    output: &'a str,
}
impl<'a> Unary<'a> {
    fn new(input: Input<'a>, output: &'a str) -> Self {
        Unary { input, output }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Binary<'a> {
    input1: Input<'a>,
    input2: Input<'a>,
    output: &'a str,
}
impl<'a> Binary<'a> {
    fn new(input1: Input<'a>, input2: Input<'a>, output: &'a str) -> Self {
        Binary {
            input1,
            input2,
            output,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Element<'a> {
    Buffer(Unary<'a>),
    Not(Unary<'a>),
    ShiftLeft(Unary<'a>, usize),
    ShiftRight(Unary<'a>, usize),
    And(Binary<'a>),
    Or(Binary<'a>),
}
impl<'a> Parseable<'a> for Element<'a> {
    fn parser(input: &'a str) -> NomParseResult<Self> {
        use Element::*;

        /// A nom parser for the input/output separator
        fn io_sep<'a, E>(input: &'a str) -> IResult<&str, (), E>
        where
            E: nom::error::ParseError<&'a str>,
        {
            value((), separator(tag("->")))(input)
        }

        /// A nom parser for the shift element
        fn shift<'a, E>(
            keyword: &'static str,
            mapper: fn(Unary<'a>, usize) -> Element<'a>,
        ) -> impl FnMut(&'a str) -> IResult<&'a str, Element<'a>, E>
        where
            E: nom::error::ParseError<&'a str>,
        {
            map(
                separated_pair(
                    separated_pair(Input::parser, separator(tag(keyword)), digit1),
                    io_sep,
                    alpha1,
                ),
                move |((i, ds), os)| mapper(Unary::new(i, os), ds.parse().unwrap()),
            )
        }

        /*
        /// A nom parser for a binary operation
        fn binary<'a, E>(
            keyword: &'static str,
            mapper: fn(Binary<'a>) -> Element<'a>,
        ) -> impl FnMut(&'a str) -> IResult<&'a str, Element<'a>, E>
        where
            E: nom::error::ParseError<&'a str>,
        {
            map(
                separated_pair(
                    separated_pair(Input::parser, separator(tag(keyword)), Input::parser),
                    io_sep,
                    alpha1,
                ),
                move |((i1s, i2s), os)| mapper(Binary::new(i1s, i2s, os)),
            )
        }*/
        alt((
            map(separated_pair(Input::parser, io_sep, alpha1), |(i, os)| {
                Buffer(Unary::new(i, os))
            }),
            map(
                separated_pair(preceded(tag("NOT "), Input::parser), io_sep, alpha1),
                |(i, os)| Not(Unary::new(i, os)),
            ),
            /*shift("LSHIFT", |u, a| ShiftLeft(u, a)),
            shift("RSHIFT", |u, a| ShiftRight(u, a)),
            binary("AND", |b| And(b)),
            binary("OR", |b| Or(b)),*/
        ))(input.trim())
    }
}
impl Element<'_> {
    fn output(&self) -> &str {
        use Element::*;
        match self {
            Buffer(u) => u.output,
            Not(u) => u.output,
            ShiftLeft(u, _) => u.output,
            ShiftRight(u, _) => u.output,
            And(b) => b.output,
            Or(b) => b.output,
        }
    }
}

#[derive(Debug)]
struct Circuit<'a> {
    elements: Box<[Element<'a>]>,
}
impl<'a> Circuit<'a> {
    fn from_str(s: &'a str) -> AocResult<Self> {
        let elements = Element::gather(s.lines())?.into_boxed_slice();

        // Ensure that no wire is connected to multiple outputs
        for element in elements.iter() {
            let wire = element.output();
            if FilterCount::<_, usize>::filter_count(elements.iter(), |e| e.output() == wire) > 1 {
                return Err(AocError::InvalidInput(
                    format!("The wire '{}' is connected to multiple outputs", wire).into(),
                ));
            }
        }

        Ok(Circuit { elements })
    }

    fn determine_signal(&self, wire: &str) -> AocResult<u16> {
        use Element::*;
        let element = self
            .elements
            .iter()
            .find(|e| e.output() == wire)
            .ok_or_else(|| {
                AocError::Process(format!("Wire '{}' not connected to an output", wire).into())
            })?;

        todo!()
        /*
        let det_input = |input: &Input| -> AocResult<u16> {
            Ok(match input {
                Input::Value(v) => v,
                Input::Wire(w) => self.determine_signal(w)?,
            })
        };

        Ok(match element {
            Set(_, v) => *v,
            Not(u) => !self.determine_signal(u.input)?,
            ShiftLeft(u, a) => self.determine_signal(u.input)? << a,
            ShiftRight(u, a) => self.determine_signal(u.input)? >> a,
            And(b) => self.determine_signal(b.input1)? & self.determine_signal(b.input2)?,
            Or(b) => self.determine_signal(b.input1)? | self.determine_signal(b.input2)?,
        })*/
    }
}

pub const SOLUTION: Solution = Solution {
    day: 7,
    name: "Some Assembly Required",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let circuit = Circuit::from_str(input)?;

            // Process
            Ok(Answer::Unsigned(circuit.determine_signal("a")?.into()))
        },
    ],
};
