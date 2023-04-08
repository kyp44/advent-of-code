use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "D2FE28";
            answers = vec![Some(Unsigned(6)), None];
        }
        example {
            input = "38006F45291200";
            answers = vec![Some(Unsigned(9)), None];
        }
        example {
            input = "EE00D40C823060";
            answers = vec![Some(Unsigned(14)), None];
        }
        example {
            input = "8A004A801A8002F478";
            answers = vec![Some(Unsigned(16)), None];
        }
        example {
            input = "620080001611562C8802118E34";
            answers = vec![Some(Unsigned(12)), None];
        }
        example {
            input = "C0015000016115A2E0802F182340";
            answers = vec![Some(Unsigned(23)), None];
        }
        example {
            input = "A0016C880162017C3686B18A3D4780";
            answers = vec![Some(Unsigned(31)), None];
        }
        example {
            input = "C200B40A82";
            answers = vec![None, Some(Unsigned(3))];
        }
        example {
            input = "04005AC33890";
            answers = vec![None, Some(Unsigned(54))];
        }
        example {
            input = "880086C3E88112";
            answers = vec![None, Some(Unsigned(7))];
        }
        example {
            input = "CE00C43D881120";
            answers = vec![None, Some(Unsigned(9))];
        }
        example {
            input = "D8005AC2A8F0";
            answers = vec![None, Some(Unsigned(1))];
        }
        example {
            input = "F600BC2D8F";
            answers = vec![None, Some(Unsigned(0))];
        }
        example {
            input = "9C005AC2F8F0";
            answers = vec![None, Some(Unsigned(0))];
        }
        example {
            input = "9C0141080250320F1802104A08";
            answers = vec![None, Some(Unsigned(1))];
        }
        actual_answers = vec![Unsigned(963), Unsigned(1549026292886)];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use bitbuffer::{BigEndian, BitReadBuffer, BitWriteStream};
    use hex::decode;
    use nom::{bits::complete::take, multi::count, Finish};

    /// An operation.
    #[derive(Debug)]
    enum Operation {
        /// Adds the values of its sub-packets.
        Sum,
        /// Multiples the values of its sub-packets.
        Product,
        /// The minimum value of its sub-packets.
        Minimum,
        /// The maximum value of its sub-packets.
        Maximum,
        /// 1 if the value of the first of its two sub-packets is greater than the
        /// value of the second, and 0 otherwise.
        GreaterThan,
        /// 1 if the value of the first of its two sub-packets is less than the
        /// value of the second, and 0 otherwise.
        LessThan,
        /// 1 if the value of the first of its two sub-packets is equal to the
        /// value of the second, and 0 otherwise.
        EqualTo,
    }
    impl Operation {
        /// Creates the operation from its operation number.
        fn from_value(v: u8) -> Option<Self> {
            match v {
                0 => Some(Self::Sum),
                1 => Some(Self::Product),
                2 => Some(Self::Minimum),
                3 => Some(Self::Maximum),
                5 => Some(Self::GreaterThan),
                6 => Some(Self::LessThan),
                7 => Some(Self::EqualTo),
                _ => None,
            }
        }
    }

    /// The type of a packet, which can be parsed from raw bytes.
    #[derive(Debug)]
    enum PacketType {
        /// A literal value with the value.
        Literal(u64),
        /// An operation with the operation and list of sub-packets.
        Operator(Operation, Box<[Packet]>),
    }
    impl PacketType {
        /// This is a [`nom`] parser for the packet type.
        fn parser(i: BitInput) -> NomParseResult<BitInput, (Self, usize)> {
            let (i, type_id) = take(3usize)(i)?;
            let mut taken_bits = 3;
            Ok(match type_id {
                4u8 => {
                    // Literal, so extract the value
                    let mut bytes = Vec::new();
                    let mut write_stream = BitWriteStream::new(&mut bytes, BigEndian);
                    let mut input = i;
                    let mut num_bits = 0;

                    // Read each nibble until we get the terminating nibble
                    loop {
                        let (i, last): (BitInput, u8) = take(1usize)(input)?;
                        let (i, nibble): (BitInput, u8) = take(4usize)(i)?;
                        write_stream.write_int(nibble, 4).unwrap();
                        num_bits += 4;
                        taken_bits += 5;
                        input = i;
                        if last == 0 {
                            break;
                        }
                    }

                    // Read complete literal value
                    let read_buffer = BitReadBuffer::new(&bytes, BigEndian);
                    (
                        input,
                        (
                            Self::Literal(read_buffer.read_int(0, num_bits).unwrap()),
                            taken_bits,
                        ),
                    )
                }
                _ => {
                    // Operator, so first determine operation
                    let operation = Operation::from_value(type_id)
                        .ok_or_else(|| NomParseError::nom_err_for_bits("Unknown operator"))?;

                    // Now get length type ID and packets
                    let (i, length_type_id): (BitInput, u8) = take(1usize)(i)?;
                    taken_bits += 1;

                    if length_type_id == 0 {
                        // Total subsequent packet length is in the next 15 bits
                        let (mut i, mut total_bits_left): (BitInput, usize) = take(15usize)(i)?;
                        taken_bits += 15 + total_bits_left;
                        let mut packets = Vec::new();

                        while total_bits_left > 0 {
                            let (inp, (packet, num_bits)) = Packet::parser(i)?;

                            if num_bits > total_bits_left {
                                return Err(NomParseError::nom_err_for_bits(
                                    "Packet took more bits than expected",
                                ));
                            }
                            i = inp;
                            total_bits_left -= num_bits;
                            packets.push(packet)
                        }

                        (
                            i,
                            (
                                Self::Operator(operation, packets.into_boxed_slice()),
                                taken_bits,
                            ),
                        )
                    } else {
                        // Number of subsequent packets is in the next 11 bits
                        let (i, num_packets): (BitInput, u16) = take(11usize)(i)?;
                        taken_bits += 11;
                        let (i, packets) = count(Packet::parser, num_packets.into())(i)?;
                        taken_bits += packets.iter().map(|t| t.1).sum::<usize>();
                        (
                            i,
                            (
                                Self::Operator(
                                    operation,
                                    packets.into_iter().map(|t| t.0).collect(),
                                ),
                                taken_bits,
                            ),
                        )
                    }
                }
            })
        }

        /// Returns the sum of version numbers of the sub packets, or zero for literals.
        fn version_sum(&self) -> u64 {
            match self {
                PacketType::Literal(_) => 0,
                PacketType::Operator(_, packets) => packets.iter().map(|p| p.version_sum()).sum(),
            }
        }

        /// Evaluates the value of this operation, or just the literal value in
        /// the case of a literal.
        fn evaluate(&self) -> AocResult<u64> {
            Ok(match self {
                PacketType::Literal(v) => *v,
                PacketType::Operator(operation, packets) => {
                    /// This is an internal function of [`PacketType::evaluate`] that just creates an error
                    /// for an operation that is missing operands.
                    fn min_one_err(operation: &Operation) -> AocError {
                        AocError::Process(
                            format!("Operation {operation:?} must have at least one operand")
                                .into(),
                        )
                    }
                    let exactly_two = |operation: &Operation| -> AocResult<(u64, u64)> {
                        if packets.len() != 2 {
                            Err(AocError::Process(
                                format!("Operation {operation:?} must have exactly two operands")
                                    .into(),
                            ))
                        } else {
                            Ok((packets[0].evaluate()?, packets[1].evaluate()?))
                        }
                    };

                    let values = || -> AocResult<Vec<u64>> {
                        packets.iter().map(|p| p.evaluate()).collect()
                    };

                    match *operation {
                        Operation::Sum => values()?.into_iter().sum(),
                        Operation::Product => values()?.into_iter().product(),
                        Operation::Minimum => values()?
                            .into_iter()
                            .min()
                            .ok_or_else(|| min_one_err(operation))?,
                        Operation::Maximum => values()?
                            .into_iter()
                            .max()
                            .ok_or_else(|| min_one_err(operation))?,
                        Operation::GreaterThan => {
                            let vals = exactly_two(operation)?;
                            u64::from(vals.0 > vals.1)
                        }
                        Operation::LessThan => {
                            let vals = exactly_two(operation)?;
                            u64::from(vals.0 < vals.1)
                        }
                        Operation::EqualTo => {
                            let vals = exactly_two(operation)?;
                            u64::from(vals.0 == vals.1)
                        }
                    }
                }
            })
        }
    }

    /// A complete packet, which can be parsed from raw bytes.
    #[derive(Debug)]
    pub struct Packet {
        /// The version number of the packet.
        version: u8,
        /// The type of the packet.
        packet_type: PacketType,
    }
    impl Packet {
        /// This is a [`nom`] parser for the packet.
        fn parser(i: BitInput) -> NomParseResult<BitInput, (Self, usize)> {
            let (i, version) = take(3usize)(i)?;
            let (i, (packet_type, type_bits)) = PacketType::parser(i)?;

            Ok((
                i,
                (
                    Self {
                        version,
                        packet_type,
                    },
                    3 + type_bits,
                ),
            ))
        }

        /// Returns the version number for the packet.
        pub fn version_sum(&self) -> u64 {
            self.packet_type.version_sum() + u64::from(self.version)
        }

        /// Evaluates the value of the packet.
        pub fn evaluate(&self) -> AocResult<u64> {
            self.packet_type.evaluate()
        }
    }
    impl FromStr for Packet {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let bytes =
                decode(s.trim()).map_err(|_| AocError::InvalidInput("invalid hex input".into()))?;
            let (packet, _) = Self::parser((&bytes, 0)).finish().discard_input()?;
            Ok(packet)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 16,
    name: "Packet Decoder",
    // TODO: Look for `preprocessor: None` so see if any preprocessing opportunities were missed.
    preprocessor: Some(|input| Ok(Box::new(Packet::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<Packet>()?.version_sum().into())
        },
        // Part two
        |input| {
            // Process
            Ok(input.expect_data::<Packet>()?.evaluate()?.into())
        },
    ],
};
