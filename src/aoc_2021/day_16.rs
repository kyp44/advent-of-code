use bitbuffer::{BigEndian, BitReadBuffer, BitWriteStream};
use hex::decode;
use nom::{bits::complete::take, multi::count, IResult};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(123)],
    "D2FE28",
    vec![6u64].answer_vec(),
    "38006F45291200",
    vec![123u64].answer_vec(),
    "EE00D40C823060",
    vec![123u64].answer_vec(),
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

type BitInput<'a> = (&'a [u8], usize);

#[derive(Debug)]
enum PacketType {
    Literal(u64),
    Operator(Box<[Packet]>),
}
impl PacketType {
    fn parser(i: BitInput) -> IResult<BitInput, Self> {
        let (i, type_id) = take(3usize)(i)?;
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
                    input = i;
                    if last == 0 {
                        break;
                    }
                }

                // Read complete literal value
                let read_buffer = BitReadBuffer::new(&bytes, BigEndian);
                (
                    input,
                    Self::Literal(read_buffer.read_int(0, num_bits).unwrap()),
                )
            }
            _ => {
                // Get length type ID
                let (i, length_type_id): (BitInput, u8) = take(1usize)(i)?;

                if length_type_id == 0 {
                    // Total subsequent packet length is in the next 15 bits
                    let (i, total_bits): (BitInput, u16) = take(15usize)(i)?;

                    (i, Self::Literal(0))
                } else {
                    // Number of subsequent packets is in the next 11 bits
                    let (i, num_packets): (BitInput, u16) = take(11usize)(i)?;
                    let (i, packets) = count(Packet::parser, num_packets.into())(i)?;
                    (i, Self::Operator(packets.into_boxed_slice()))
                }
            }
        })
    }
}

#[derive(Debug)]
struct Packet {
    version: u8,
    packet_type: PacketType,
}
impl Packet {
    fn parser(i: BitInput) -> IResult<BitInput, Self> {
        let (i, version) = take(3usize)(i)?;
        let (i, packet_type) = PacketType::parser(i)?;
        Ok((
            i,
            Self {
                version,
                packet_type,
            },
        ))
    }
}

fn hex_decode(i: &str) -> AocResult<Vec<u8>> {
    decode(i).map_err(|_| AocError::InvalidInput("invalid hex input".into()))
}

pub const SOLUTION: Solution = Solution {
    day: 16,
    name: "Packet Decoder",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let bytes = hex_decode(input)?;
            let packet = Packet::parser((&bytes, 0));

            println!("TODO: {:?}", packet);

            // Process
            Ok(0u64.into())
        },
    ],
};
