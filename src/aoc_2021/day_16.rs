use hex::{decode, decode_to_slice};
use nom::{bits::complete::take, IResult};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(123)],
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

enum PacketType {
    Literal(u64),
    Operator(Box<[Packet]>),
}

struct Packet {
    version: u8,
    packet_type: PacketType,
}
impl Packet {
    fn parser(i: BitInput) -> IResult<BitInput, Self> {
        let (i, version) = take(3usize)(i)?;
        Ok((
            i,
            Self {
                version,
                packet_type: PacketType::Literal(0),
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

            // Process
            Ok(0u64.into())
        },
    ],
};
