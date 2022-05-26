use std::{cell::RefCell, collections::HashMap, convert::TryInto};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::alpha1,
    combinator::{map, value},
    sequence::{preceded, separated_pair},
    IResult,
};

use crate::aoc::{parse::separated, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(46065), Unsigned(14134)],
    "123 -> x
456 -> y
x AND y -> d
x OR y -> e
x LSHIFT 2 -> f
y RSHIFT 2 -> g
NOT x -> h
NOT y -> i
f -> a",
    vec![492u64].answer_vec()
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Input<'a> {
    Value(u16),
    Wire(&'a str),
}
impl<'a> Parseable<'a> for Input<'a> {
    fn parser(input: &'a str) -> NomParseResult<&str, Self> {
        alt((
            map(nom::character::complete::u16, Input::Value),
            map(alpha1, Input::Wire),
        ))(input)
    }
}

#[derive(Debug, PartialEq, Eq, new)]
struct Unary<'a> {
    input: Input<'a>,
    output: &'a str,
}

#[derive(Debug, PartialEq, Eq, new)]
struct Binary<'a> {
    input1: Input<'a>,
    input2: Input<'a>,
    output: &'a str,
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
    fn parser(input: &'a str) -> NomParseResult<&str, Self> {
        /// A nom parser for the input/output separator
        fn io_sep<'a, E>(input: &'a str) -> IResult<&str, (), E>
        where
            E: nom::error::ParseError<&'a str>,
        {
            value((), separated(tag("->")))(input)
        }

        /// A nom parser for the shift element
        fn shift<'a>(
            keyword: &'static str,
            mapper: fn(Unary<'a>, usize) -> Element<'a>,
        ) -> impl FnMut(&'a str) -> IResult<&'a str, Element<'a>, NomParseError> {
            map(
                separated_pair(
                    separated_pair(
                        Input::parser,
                        separated(tag(keyword)),
                        nom::character::complete::u64,
                    ),
                    io_sep,
                    alpha1,
                ),
                move |((i, d), os)| mapper(Unary::new(i, os), d.try_into().unwrap()),
            )
        }

        /// A nom parser for a binary operation
        fn binary<'a>(
            keyword: &'static str,
            mapper: fn(Binary<'a>) -> Element<'a>,
        ) -> impl FnMut(&'a str) -> IResult<&'a str, Element<'a>, NomParseError> {
            map(
                separated_pair(
                    separated_pair(Input::parser, separated(tag(keyword)), Input::parser),
                    io_sep,
                    alpha1,
                ),
                move |((i1s, i2s), os)| mapper(Binary::new(i1s, i2s, os)),
            )
        }

        alt((
            map(separated_pair(Input::parser, io_sep, alpha1), |(i, os)| {
                Element::Buffer(Unary::new(i, os))
            }),
            map(
                separated_pair(preceded(tag("NOT "), Input::parser), io_sep, alpha1),
                |(i, os)| Element::Not(Unary::new(i, os)),
            ),
            shift("LSHIFT", Element::ShiftLeft),
            shift("RSHIFT", Element::ShiftRight),
            binary("AND", Element::And),
            binary("OR", Element::Or),
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
    wire_values: RefCell<HashMap<&'a str, u16>>,
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

        Ok(Circuit {
            elements,
            wire_values: RefCell::new(HashMap::new()),
        })
    }

    fn determine_signal<'b>(&'b mut self, wire: &'a str) -> AocResult<u16> {
        // Recursive internal function
        fn det_sig<'a: 'b, 'b>(
            wire_values: &'b RefCell<HashMap<&'a str, u16>>,
            elements: &'b [Element<'a>],
            wire: &'a str,
        ) -> AocResult<u16> {
            if let Some(val) = wire_values.borrow().get(wire) {
                //println!("Found wire '{}' in lookup table", wire);
                return Ok(*val);
            }
            let element = elements
                .iter()
                .find(|e| e.output() == wire)
                .ok_or_else(|| Circuit::wire_error(wire))?;

            let det_input = |input: &Input<'a>| -> AocResult<u16> {
                Ok(match input {
                    Input::Value(v) => *v,
                    Input::Wire(w) => det_sig(wire_values, elements, w)?,
                })
            };

            //println!("Determining wire {}: {:?}", wire, element);
            use Element::*;
            let val = match element {
                Buffer(u) => det_input(&u.input)?,
                Not(u) => !det_input(&u.input)?,
                ShiftLeft(u, a) => det_input(&u.input)? << a,
                ShiftRight(u, a) => det_input(&u.input)? >> a,
                And(b) => det_input(&b.input1)? & det_input(&b.input2)?,
                Or(b) => det_input(&b.input1)? | det_input(&b.input2)?,
            };

            wire_values.borrow_mut().insert(wire, val);
            Ok(val)
        }

        det_sig(&self.wire_values, &self.elements, wire)
    }

    fn wire_error(wire: &str) -> AocError {
        AocError::Process(format!("Wire '{}' not connected to an output", wire).into())
    }

    fn override_wire<'b>(&'b mut self, wire: &'a str, value: u16) -> AocResult<()> {
        // Change the wire to the specified value
        let element = self
            .elements
            .iter_mut()
            .find(|e| e.output() == wire)
            .ok_or_else(|| Self::wire_error(wire))?;
        *element = Element::Buffer(Unary::new(Input::Value(value), wire));

        // Now reset known wires
        self.wire_values.borrow_mut().clear();

        Ok(())
    }
}

pub const SOLUTION: Solution = Solution {
    day: 7,
    name: "Some Assembly Required",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let mut circuit = Circuit::from_str(input)?;

            // Process
            Ok(Answer::Unsigned(circuit.determine_signal("a")?.into()))
        },
        // Part b)
        |input| {
            // Generation
            let mut circuit = Circuit::from_str(input)?;

            // Find part a) solution an override
            let a = circuit.determine_signal("a")?;
            circuit.override_wire("b", a)?;

            // Process
            Ok(Answer::Unsigned(circuit.determine_signal("a")?.into()))
        },
    ],
};
