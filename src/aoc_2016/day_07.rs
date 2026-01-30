use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "abba[mnop]qrst
abcd[bddb]xyyx
aaaa[qwer]tyui
ioxxoj[asdfgh]zxcvbn";
            answers = unsigned![2, 0];
        }
        example {
            input = "aba[bab]xyz
xyx[xyx]xyx
aaa[kek]eke
zazbz[bzb]cdb";
            answers = unsigned![0, 3];
        }
        actual_answers = unsigned![105, 258];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use nom::{
        branch::alt, bytes::complete::tag, character::complete::alpha1, combinator::map,
        multi::many1, sequence::delimited,
    };
    use std::{collections::HashSet, ops::Neg};

    /// A generic sequence within an [`IpAddress`].
    #[derive(Debug)]
    enum AddressSeq<T> {
        /// A sequence in a supernet section of the IP address.
        Supernet(T),
        /// A sequence in a hypernet section of the IP address
        /// (surround by square brackets).
        Hypernet(T),
    }
    impl<T: AsRef<str>> AddressSeq<T> {
        /// Returns the sequence string.
        pub fn as_str(&self) -> &str {
            match self {
                AddressSeq::Supernet(s) => s.as_ref(),
                AddressSeq::Hypernet(s) => s.as_ref(),
            }
        }

        /// Processes a pattern in the in the sequence and returns each
        /// pattern found in the sequence in order.
        ///
        /// If the pattern is not found, an empty [`Vec`] is returned.
        pub fn process_pattern<P>(&self) -> Vec<AddressSeq<P>>
        where
            P: Pattern,
            [(); P::LEN]:,
        {
            let build = |f: fn(P) -> AddressSeq<P>| -> Vec<AddressSeq<P>> {
                let chars: Vec<_> = self.as_str().chars().collect();
                chars
                    .windows(P::LEN)
                    .filter_map(|cs: &[char]| P::from_chars(cs.try_into().unwrap()).map(f))
                    .collect()
            };

            match self {
                AddressSeq::Supernet(_) => build(AddressSeq::Supernet),
                AddressSeq::Hypernet(_) => build(AddressSeq::Hypernet),
            }
        }
    }

    /// Trait for possible statuses of an [`IpAddress`] object.
    pub trait AddressStatus {}

    /// An [`IpAddress`] that has just been parsed from text.
    #[derive(Debug)]
    pub struct Raw {
        /// The complete string sequences of every section of the IP address.
        sequences: Vec<AddressSeq<String>>,
    }
    impl AddressStatus for Raw {}

    /// Trait for patterns to find in address sequences.
    trait Pattern: Sized {
        /// The length of the pattern in characters.
        const LEN: usize;

        /// Returns the [`Pattern`] if the sequences of `chars` matches
        /// the pattern and `None` otherwise.
        fn from_chars(chars: [char; Self::LEN]) -> Option<Self>;
    }

    /// The ABBA pattern from part one.
    #[derive(Debug)]
    struct AbbaPattern {
        /// The specific characters of the pattern.
        _chars: [char; 4],
    }
    impl Pattern for AbbaPattern {
        const LEN: usize = 4;

        fn from_chars(chars: [char; Self::LEN]) -> Option<Self> {
            (chars[0] != chars[1] && chars[0] == chars[3] && chars[1] == chars[2])
                .then_some(Self { _chars: chars })
        }
    }

    /// The ABA pattern from part two.
    #[derive(Debug, PartialEq, Eq, Hash)]
    struct AbaPattern {
        /// The specific characters of the pattern.
        chars: [char; 3],
    }
    impl Pattern for AbaPattern {
        const LEN: usize = 3;

        fn from_chars(chars: [char; Self::LEN]) -> Option<Self> {
            (chars[0] != chars[1] && chars[0] == chars[2]).then_some(Self { chars })
        }
    }
    impl Neg for &AbaPattern {
        type Output = AbaPattern;

        /// Returns the inverse pattern BAB.
        fn neg(self) -> Self::Output {
            let a = self.chars[0];
            let b = self.chars[1];
            AbaPattern { chars: [b, a, b] }
        }
    }

    /// An [`IpAddress`] that has been fully processed with the patterns pulled out.
    #[derive(Debug)]
    pub struct Processed {
        /// The ABBA patterns for part one.
        abba_sequences: Vec<AddressSeq<AbbaPattern>>,
        /// The ABA patterns for part two.
        aba_sequences: Vec<AddressSeq<AbaPattern>>,
    }
    impl AddressStatus for Processed {}

    /// An IP address with a particular [`AddressStatus`].
    ///
    /// A [`Raw`] address can be parsed from text input.
    #[derive(Debug)]
    struct IpAddress<S: AddressStatus> {
        /// The current status.
        status: S,
    }
    impl Parsable<'_> for IpAddress<Raw> {
        fn parser(input: &'_ str) -> NomParseResult<&'_ str, Self> {
            map(
                trim(
                    true,
                    many1(alt((
                        map(delimited(tag("["), alpha1, tag("]")), |s: &str| {
                            AddressSeq::Hypernet(s.into())
                        }),
                        map(alpha1, |s: &str| AddressSeq::Supernet(s.into())),
                    ))),
                ),
                |sequences| Self {
                    status: Raw { sequences },
                },
            )
            .parse(input)
        }
    }
    impl IpAddress<Raw> {
        /// Process the [`Raw`] address.
        pub fn process(self) -> AocResult<IpAddress<Processed>> {
            fn build<P>(addr: &IpAddress<Raw>) -> Vec<AddressSeq<P>>
            where
                P: Pattern,
                [(); P::LEN]:,
            {
                addr.status
                    .sequences
                    .iter()
                    .flat_map(|seq| seq.process_pattern::<P>())
                    .collect()
            }

            Ok(IpAddress {
                status: Processed {
                    abba_sequences: build::<AbbaPattern>(&self),
                    aba_sequences: build::<AbaPattern>(&self),
                },
            })
        }
    }
    impl IpAddress<Processed> {
        /// Returns whether or not the address supports TLS for part one.
        pub fn supports_tls(&self) -> bool {
            let mut super_abba = false;
            for seq in self.status.abba_sequences.iter() {
                match seq {
                    AddressSeq::Supernet(_) => super_abba = true,
                    AddressSeq::Hypernet(_) => return false,
                }
            }

            super_abba
        }

        /// Returns whether or not address supports SSL for part two.
        pub fn supports_ssl(&self) -> bool {
            // First split out the sections, which is a little obnoxious
            let mut supernets = HashSet::new();
            let mut hypernets = HashSet::new();

            for seq in self.status.aba_sequences.iter() {
                match seq {
                    AddressSeq::Supernet(aba) => supernets.insert(aba),
                    AddressSeq::Hypernet(aba) => hypernets.insert(aba),
                };
            }

            // Now, for each supernet ABA see if the corresponding BAB is in the hypernet
            for aba in supernets.into_iter() {
                if hypernets.contains(&-aba) {
                    return true;
                }
            }

            false
        }
    }

    /// A collection of IP addresses.
    ///
    /// Can be parsed and processed from text input.
    pub struct IpAddresses {
        /// The list of addresses.
        addresses: Vec<IpAddress<Processed>>,
    }
    impl FromStr for IpAddresses {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                addresses: IpAddress::<Raw>::gather(s.lines())?
                    .into_iter()
                    .map(|a| a.process())
                    .collect::<AocResult<_>>()?,
            })
        }
    }
    impl IpAddresses {
        /// Counts the number of addresses that support TLS for part one.
        pub fn num_support_tls(&self) -> u64 {
            self.addresses.iter().filter_count(|a| a.supports_tls())
        }

        /// Counts the number of addresses that support SSL for part two.
        pub fn num_support_ssl(&self) -> u64 {
            self.addresses.iter().filter_count(|a| a.supports_ssl())
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 7,
    name: "Internet Protocol Version 7",
    preprocessor: Some(|input| Ok(Box::new(IpAddresses::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<IpAddresses>()?.num_support_tls().into())
        },
        // Part two
        |input| {
            // Process
            Ok(input.expect_data::<IpAddresses>()?.num_support_ssl().into())
        },
    ],
};
