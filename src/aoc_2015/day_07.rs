use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "123 -> x
456 -> y
x AND y -> d
x OR y -> e
x LSHIFT 2 -> f
y RSHIFT 2 -> g
NOT x -> h
NOT y -> i
f -> a";
            answers = unsigned![492];
        }
        actual_answers = unsigned![46065, 14134];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::separated;
    use derive_new::new;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::alpha1,
        combinator::{map, value},
        sequence::{preceded, separated_pair},
        IResult,
    };
    use std::{collections::HashMap, convert::TryInto};

    /// An input to a bitwise component or wire.
    #[derive(Debug, PartialEq, Eq)]
    enum Input<'a> {
        /// A numeric value.
        Value(u16),
        /// A wire with a name.
        Wire(&'a str),
    }
    impl<'a> Parsable<'a> for Input<'a> {
        fn parser(input: &'a str) -> NomParseResult<&str, Self> {
            alt((
                map(nom::character::complete::u16, Input::Value),
                map(alpha1, Input::Wire),
            ))(input)
        }
    }

    /// A unary bitwise component or a wire.
    #[derive(Debug, PartialEq, Eq, new)]
    struct Unary<'a> {
        /// Input.
        input: Input<'a>,
        /// Output wire name.
        output: &'a str,
    }

    /// A binary bitwise component.
    #[derive(Debug, PartialEq, Eq, new)]
    struct Binary<'a> {
        /// Input 1.
        input1: Input<'a>,
        /// Input 2.
        input2: Input<'a>,
        /// Output wire name.
        output: &'a str,
    }

    /// A bitwise component that can be parsed from text input.
    #[derive(Debug, PartialEq, Eq)]
    enum Element<'a> {
        /// A simple buffer.
        Buffer(Unary<'a>),
        /// Bitwise complimenter.
        Not(Unary<'a>),
        /// Bitwise left shifter.
        ShiftLeft(Unary<'a>, usize),
        /// Bitwise reft shifter.
        ShiftRight(Unary<'a>, usize),
        /// Bitwise AND gate.
        And(Binary<'a>),
        /// Bitwise OR gate.
        Or(Binary<'a>),
    }
    impl<'a> Parsable<'a> for Element<'a> {
        fn parser(input: &'a str) -> NomParseResult<&str, Self> {
            /// This is a [`nom`] parser for the input/output separator.
            fn io_sep<'a, E>(input: &'a str) -> IResult<&str, (), E>
            where
                E: nom::error::ParseError<&'a str>,
            {
                value((), separated(tag("->")))(input)
            }

            /// This is a [`nom`] parser for the shift element.
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

            /// This is a [`nom`] parser for a binary operation.
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
        /// Provides the output wire name for the component since all components have a single output.
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

    /// A complete bitwise circuit that can be parsed from text input.
    #[derive(Debug)]
    pub struct Circuit<'a> {
        /// List of the elements that the circuit comprises.
        elements: Box<[Element<'a>]>,
        /// Set of all the wire names within the circuit.
        wire_values: HashMap<&'a str, u16>,
    }
    impl<'a> Circuit<'a> {
        /// Parses the circuit from input text.
        pub fn from_str(s: &'a str) -> AocResult<Self> {
            let elements = Element::gather(s.lines())?.into_boxed_slice();

            // Ensure that no wire is connected to multiple outputs
            for element in elements.iter() {
                let wire = element.output();
                if elements
                    .iter()
                    .filter_count::<usize>(|e| e.output() == wire)
                    > 1
                {
                    return Err(AocError::InvalidInput(
                        format!("The wire '{wire}' is connected to multiple outputs").into(),
                    ));
                }
            }

            Ok(Circuit {
                elements,
                wire_values: HashMap::new(),
            })
        }

        /// Determines the resulting value on a wire when the circuit is connected.
        pub fn determine_signal<'b>(&'b mut self, wire: &'a str) -> AocResult<u16> {
            /// This is an internal function for [`Circuit::determine_signal`] to determine
            /// the value on a wire.
            fn det_sig<'a: 'b, 'b>(
                wire_values: &'b mut HashMap<&'a str, u16>,
                elements: &'b [Element<'a>],
                wire: &'a str,
            ) -> AocResult<u16> {
                if let Some(val) = wire_values.get(wire) {
                    //println!("Found wire '{}' in lookup table", wire);
                    return Ok(*val);
                }
                let element = elements
                    .iter()
                    .find(|e| e.output() == wire)
                    .ok_or_else(|| Circuit::wire_error(wire))?;

                let mut det_input = |input: &Input<'a>| -> AocResult<u16> {
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

                wire_values.insert(wire, val);
                Ok(val)
            }

            det_sig(&mut self.wire_values, &self.elements, wire)
        }

        /// Generates an error indicating that a wire is not connected to an output.
        fn wire_error(wire: &str) -> AocError {
            AocError::Process(format!("Wire '{wire}' not connected to an output").into())
        }

        /// Overrides the value on a wire in the circuit.
        pub fn override_wire<'b>(&'b mut self, wire: &'a str, value: u16) -> AocResult<()> {
            // Change the wire to the specified value
            let element = self
                .elements
                .iter_mut()
                .find(|e| e.output() == wire)
                .ok_or_else(|| Self::wire_error(wire))?;
            *element = Element::Buffer(Unary::new(Input::Value(value), wire));

            // Now reset known wires
            self.wire_values.clear();

            Ok(())
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 7,
    name: "Some Assembly Required",
    // NOTE: Circuit keeps references to input, so we cannot use a pre-processor.
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let mut circuit = Circuit::from_str(input.expect_text()?)?;

            // Process
            Ok(Answer::Unsigned(circuit.determine_signal("a")?.into()))
        },
        // Part two
        |input| {
            // Generation
            let mut circuit = Circuit::from_str(input.expect_text()?)?;

            // Find Part one solution an override
            let a = circuit.determine_signal("a")?;
            circuit.override_wire("b", a)?;

            // Process
            Ok(Answer::Unsigned(circuit.determine_signal("a")?.into()))
        },
    ],
};
