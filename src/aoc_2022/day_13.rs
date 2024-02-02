use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]";
            answers = unsigned![13, 140];
        }
        actual_answers = unsigned![5605, 24969];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use itertools::Itertools;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::line_ending,
        combinator::map,
        multi::separated_list0,
        sequence::{delimited, separated_pair},
    };

    /// An element of a packet, which can also be a packet itself.
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Element {
        /// Just an integer.
        Integer(u8),
        /// An ordered list of other packet elements.
        List(Box<[Element]>),
    }
    impl Element {
        /// Puts this element into a list element that contains only this element.
        pub fn put_in_list(self) -> Self {
            Self::List(Box::new([self]))
        }
    }
    impl Parsable<'_> for Element {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            alt((
                map(nom::character::complete::u8, Self::Integer),
                map(
                    delimited(tag("["), separated_list0(tag(","), Self::parser), tag("]")),
                    |v| Self::List(v.into_boxed_slice()),
                ),
            ))(input)
        }
    }
    impl PartialOrd for Element {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Element {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            match self {
                Element::Integer(ns) => match other {
                    Element::Integer(bo) => ns.cmp(bo),
                    Element::List(_) => self.clone().put_in_list().cmp(other),
                },
                Element::List(ls) => match other {
                    Element::Integer(_) => self.cmp(&other.clone().put_in_list()),
                    Element::List(lo) => ls.iter().cmp(lo.iter()),
                },
            }
        }
    }

    /// A pair of packets.
    #[derive(Debug)]
    struct PacketPair {
        /// The `left` packet.
        left: Element,
        /// The `right` packet.
        right: Element,
    }
    impl Parsable<'_> for PacketPair {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(
                    trim(false, Element::parser),
                    line_ending,
                    trim(false, Element::parser),
                ),
                |(left, right)| Self { left, right },
            )(input)
        }
    }
    impl PacketPair {
        /// Returns a two-element [`Iterator`] that iterates over the `left` and then `right`
        /// packets in succession.
        pub fn iter(&self) -> impl Iterator<Item = &Element> {
            [&self.left, &self.right].into_iter()
        }
    }

    /// The list of packet pairs.
    #[derive(Debug)]
    pub struct PacketPairs {
        /// The list of pairs.
        packet_pairs: Vec<PacketPair>,
    }
    impl FromStr for PacketPairs {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                packet_pairs: PacketPair::gather(s.split("\n\n"))?,
            })
        }
    }
    impl PacketPairs {
        /// Compares the packets of each pair, and adds the indices of those pairs that
        /// are in the correct order (part one).
        pub fn sum_of_correct_pair_indices(&self) -> u64 {
            self.packet_pairs
                .iter()
                .enumerate()
                .filter(|(_, p)| p.left < p.right)
                .map(|(i, _)| u64::try_from(i + 1).unwrap())
                .sum()
        }

        /// Sorts all of the packets as well as the divider packets, and multiplies the
        /// indices of the two divider packets in the sorted list (part two).
        pub fn decoder_key(&self) -> u64 {
            let divider_packets = [
                Element::Integer(2).put_in_list().put_in_list(),
                Element::Integer(6).put_in_list().put_in_list(),
            ];
            let mut packets = self
                .packet_pairs
                .iter()
                .flat_map(PacketPair::iter)
                .chain(divider_packets.iter())
                .collect_vec();

            packets.sort();

            packets
                .into_iter()
                .enumerate()
                .filter(|(_, e)| divider_packets.contains(e))
                .map(|(i, _)| u64::try_from(i + 1).unwrap())
                .product()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 13,
    name: "Distress Signal",
    preprocessor: Some(|input| Ok(Box::new(PacketPairs::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<PacketPairs>()?
                .sum_of_correct_pair_indices()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input.expect_data::<PacketPairs>()?.decoder_key().into())
        },
    ],
};
