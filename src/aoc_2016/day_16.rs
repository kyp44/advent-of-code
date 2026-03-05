use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "disk size: 20
    initial state: 10000";
            answers = string!["01100"];
        }
        actual_answers = string!["01110011101111011", "11001111011000111"];
    }

    #[test]
    fn expansion() {
        assert_eq!(
            BinaryString::from_str("1").unwrap().expand(),
            BinaryString::from_str("100").unwrap()
        );
        assert_eq!(
            BinaryString::from_str("0").unwrap().expand(),
            BinaryString::from_str("001").unwrap()
        );
        assert_eq!(
            BinaryString::from_str("11111").unwrap().expand(),
            BinaryString::from_str("11111000000").unwrap()
        );
        assert_eq!(
            BinaryString::from_str("111100001010").unwrap().expand(),
            BinaryString::from_str("1111000010100101011110000").unwrap()
        );
    }

    #[test]
    fn checksum() {
        assert_eq!(
            BinaryString::from_str("110010110100").unwrap().checksum(),
            BinaryString::from_str("100").unwrap(),
        );
    }
}

/// Contains solution implementation items.
mod solution {
    use std::fmt::Write;

    use super::*;
    use aoc::parse::{field_line_parser, trim};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{all_consuming, map},
        multi::many1,
    };
    use num::Integer;

    /// A string of binary values.
    ///
    /// Can be parsed from a text string of `1`s and `0`s.
    #[derive(Clone, PartialEq, Eq)]
    pub struct BinaryString(Vec<bool>);
    impl Parsable for BinaryString {
        type Parsed<'a> = Self;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(
                all_consuming(trim(
                    false,
                    many1(alt((map(tag("0"), |_| false), map(tag("1"), |_| true)))),
                )),
                Self,
            )
            .parse(input)
        }
    }
    impl BinaryString {
        /// Returns the length of the binary string.
        pub fn len(&self) -> usize {
            self.0.len()
        }

        /// Converts this into a binary string with the length truncated to
        /// `size`.
        ///
        /// If this string is no longer than `size`, then it is simply returned.
        pub fn into_truncated(self, size: usize) -> Self {
            if self.len() > size {
                Self(self.0[0..size].into())
            } else {
                self
            }
        }

        /// Returns an expanded binary string to fill more space, using the
        /// described method.
        pub fn expand(&self) -> Self {
            let mut c = self.0.clone();
            let mut b = self.0.iter().map(|b| !*b).collect::<Vec<_>>();
            b.reverse();
            c.extend(std::iter::once(false).chain(b));

            Self(c)
        }

        /// Returns a reduced binary string, applying the checksum procedure
        /// just once.
        fn checksum_reduce(&self) -> Self {
            Self(self.0.chunks(2).map(|bs| bs[0] == bs[1]).collect())
        }

        /// Returns the fully reduced checksum binary string, which is
        /// guaranteed to have an odd length.
        pub fn checksum(&self) -> Self {
            let mut reduced = self.checksum_reduce();
            while reduced.0.len().is_even() {
                reduced = reduced.checksum_reduce()
            }
            reduced
        }

        /// Returns a string representation of the binary string with `0`s and
        /// `1`s.
        pub fn as_string(&self) -> String {
            format!("{self}")
        }
    }
    impl std::fmt::Debug for BinaryString {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for bit in self.0.iter() {
                f.write_char(if *bit { '1' } else { '0' })?
            }
            Ok(())
        }
    }
    impl std::fmt::Display for BinaryString {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(self, f)
        }
    }

    /// A disk that needs to be filled.
    pub struct Disk {
        /// The total size of the disk.
        size: usize,
        /// The initial data on the disk, which likely needs to be expanded to
        /// fill it up.
        initial_state: BinaryString,
    }
    impl Parsable for Disk {
        type Parsed<'a> = Self;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(
                (
                    field_line_parser("disk size:", nom::character::complete::usize),
                    field_line_parser("initial state:", BinaryString::parser),
                ),
                |(size, initial_state)| Self {
                    size,
                    initial_state,
                },
            )
            .parse(input.trim())
        }
    }
    impl Disk {
        /// Returns the initial data that has been expanded enough times and
        /// truncated if needed to exactly fit the disk size.
        pub fn fill(&self) -> BinaryString {
            let mut content = self.initial_state.expand();

            while content.len() < self.size {
                content = content.expand();
            }
            content.into_truncated(self.size)
        }

        /// Returns the same disk, but with a different size.
        pub fn with_size(&self, size: usize) -> Self {
            Self {
                size,
                initial_state: self.initial_state.clone(),
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 16,
    name: "Dragon Checksum",
    preprocessor: Some(|input| Ok(Box::new(Disk::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Disk>()?
                .fill()
                .checksum()
                .as_string()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Disk>()?
                .with_size(35651584)
                .fill()
                .checksum()
                .as_string()
                .into())
        },
    ],
};
