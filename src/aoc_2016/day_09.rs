use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "ADVENT
            A(1x5)BC
            (3x3)XYZ
            A(2x2)BCD(2x2)EFG
            (6x1)(1x3)A
            X(8x2)(3x3)ABCY";
            answers = unsigned![6 + 7 + 9 + 11 + 6 + 18];
        }
        example {
            input = "(3x3)XYZ
            X(8x2)(3x3)ABCY
            (27x12)(20x12)(13x14)(7x10)(1x12)A
            (25x3)(3x3)ABC(2x3)XY(5x2)PQRSTX(18x9)(3x2)TWO(5x7)SEVEN";
            answers = unsigned![589, 9 + 20 + 241920 + 445];
        }
        actual_answers = unsigned![110346, 10774309173];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use either::Either;
    use nom::Finish;
    use nom::branch::alt;
    use nom::bytes::complete::{tag, take};
    use nom::character::complete::alpha1;
    use nom::combinator::{all_consuming, map};
    use nom::multi::many1;
    use std::borrow::Cow;

    /// A distinct and complete chunk of a compressed string.
    ///
    /// Can be parsed from text input.
    enum CompressedChunk<'a> {
        /// A simple string that decompresses to itself.
        String(&'a str),
        /// A marker that needs to be expanded during decompression.
        Marker {
            /// The number of times to repeat `data`.
            num_repeats: u8,
            /// The data string to repeat.
            ///
            /// Note that this may contain its own markers.
            data: &'a str,
        },
    }
    impl Parsable for CompressedChunk<'_> {
        type Parsed<'a> = CompressedChunk<'a>;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            let (rem, which) = alt((
                map(
                    trim(
                        true,
                        (
                            tag("("),
                            nom::character::complete::usize,
                            tag("x"),
                            nom::character::complete::u8,
                            tag(")"),
                        ),
                    ),
                    |(_, nc, _, nr, _)| Either::Right((nc, nr)),
                ),
                map(trim(true, alpha1), Either::Left),
            ))
            .parse(input)?;

            Ok(match which {
                Either::Left(s) => (rem, CompressedChunk::String(s)),
                Either::Right((nc, nr)) => {
                    let (rem, data) = take(nc).parse(rem)?;
                    (
                        rem,
                        CompressedChunk::Marker {
                            num_repeats: nr,
                            data,
                        },
                    )
                }
            })
        }
    }
    impl std::fmt::Debug for CompressedChunk<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::String(s) => write!(f, "{s}"),
                Self::Marker {
                    num_repeats: nr,
                    data,
                } => write!(f, "({}x{nr}){data}", data.len()),
            }
        }
    }
    impl<'a> CompressedChunk<'a> {
        /// Returns the length of the chunk after using the v1 decompression
        /// algorithm of part one.
        ///
        /// A marker is just expanded once, ignoring any markers in its `data`.
        pub fn decompressed_len_v1(&self) -> u64 {
            match self {
                CompressedChunk::String(s) => s.len(),
                CompressedChunk::Marker { num_repeats, data } => {
                    usize::from(*num_repeats) * data.len()
                }
            }
            .try_into()
            .unwrap()
        }

        /// Returns the length of the chunk after using the v2 decompression
        /// algorithm of part two.
        ///
        /// For a marker, the `data` is expanded recursively.
        pub fn decompressed_len_v2(&self) -> Result<u64, NomParseError> {
            match self {
                CompressedChunk::String(s) => Ok(s.len().try_into().unwrap()),
                CompressedChunk::Marker { num_repeats, data } => Ok(u64::from(*num_repeats)
                    * CompressedString::from_str(data).decompressed_len_v2()?),
            }
        }
    }

    /// A compressed string, guaranteed to be free of any whitespace.
    pub struct CompressedString<'a>(Cow<'a, str>);
    impl<'a> CompressedString<'a> {
        /// Removes all whitespace from a string.
        ///
        /// Returns the new [`String`] with whitespace removed or `None` if
        /// the string contained no whitespace to begin with.
        fn remove_whitespace(s: &str) -> Option<String> {
            s.chars()
                .any(char::is_whitespace)
                .then(|| s.split_whitespace().collect())
        }

        /// Creates this from a string, which will not be copied if it already
        /// contains no whitespace.
        pub fn from_str(s: &'a str) -> Self {
            Self(match Self::remove_whitespace(s) {
                Some(s) => s.into(),
                None => s.into(),
            })
        }

        /// Creates this from a string, which will be copied even if it already
        /// contains no whitespace.
        pub fn from_str_copy(s: &str) -> Self {
            Self(match Self::remove_whitespace(s) {
                Some(s) => s.into(),
                None => String::from(s).into(),
            })
        }

        /// Parses the string into chunks, but does not do so recursively for
        /// markers.
        fn parse_chunks(&self) -> Result<Vec<CompressedChunk<'_>>, NomParseError> {
            all_consuming(many1(CompressedChunk::parser))
                .parse(&self.0)
                .finish()
                .discard_input()
        }

        /// Returns the length of the string after using the v1 decompression
        /// algorithm of part one.
        pub fn decompressed_len_v1(&self) -> Result<u64, NomParseError> {
            Ok(self
                .parse_chunks()?
                .into_iter()
                .map(|c| c.decompressed_len_v1())
                .sum())
        }

        /// Returns the length of the string after using the v2 decompression
        /// algorithm of part two.
        pub fn decompressed_len_v2(&self) -> Result<u64, NomParseError> {
            itertools::process_results(
                self.parse_chunks()?
                    .into_iter()
                    .map(|c| c.decompressed_len_v2()),
                |iter| iter.sum(),
            )
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 9,
    name: "Explosives in Cyberspace",
    preprocessor: Some(|input| Ok(Box::new(CompressedString::from_str_copy(input)).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<CompressedString>()?
                .decompressed_len_v1()?
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<CompressedString>()?
                .decompressed_len_v2()?
                .into())
        },
    ],
};
