use std::str::FromStr;

use bitbuffer::{BigEndian, BitReadBuffer, BitWriteStream};
use hex::decode;
use nom::{bits::complete::take, multi::count, Finish};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(963)],
    "D2FE28",
    vec![6u64].answer_vec(),
    "38006F45291200",
    vec![9u64].answer_vec(),
    "EE00D40C823060",
    vec![14u64].answer_vec(),
    "8A004A801A8002F478",
    vec![16u64].answer_vec(),
    "620080001611562C8802118E34",
    vec![12u64].answer_vec(),
    "C0015000016115A2E0802F182340",
    vec![23u64].answer_vec(),
    "A0016C880162017C3686B18A3D4780",
    vec![31u64].answer_vec()
    }
}

#[derive(Debug)]
enum PacketType {
    Literal(u64),
    Operator(Box<[Packet]>),
}
impl PacketType {
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
                // Operator, so get length type ID
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

                    (i, (Self::Operator(packets.into_boxed_slice()), taken_bits))
                } else {
                    // Number of subsequent packets is in the next 11 bits
                    let (i, num_packets): (BitInput, u16) = take(11usize)(i)?;
                    taken_bits += 11;
                    let (i, packets) = count(Packet::parser, num_packets.into())(i)?;
                    taken_bits += packets.iter().map(|t| t.1).sum::<usize>();
                    (
                        i,
                        (
                            Self::Operator(packets.into_iter().map(|t| t.0).collect()),
                            taken_bits,
                        ),
                    )
                }
            }
        })
    }

    fn version_sum(&self) -> u64 {
        match self {
            PacketType::Literal(_) => 0,
            PacketType::Operator(packets) => packets.iter().map(|p| p.version_sum()).sum(),
        }
    }
}

#[derive(Debug)]
struct Packet {
    version: u8,
    packet_type: PacketType,
}
impl Packet {
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

    fn version_sum(&self) -> u64 {
        self.packet_type.version_sum() + u64::from(self.version)
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

pub const SOLUTION: Solution = Solution {
    day: 16,
    name: "Packet Decoder",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let packet = Packet::from_str(input)?;

            //println!("Packet: {:?}", packet);

            // Process
            Ok(packet.version_sum().into())
        },
    ],
};
