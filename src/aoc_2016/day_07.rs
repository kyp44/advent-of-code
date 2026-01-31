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

    #[derive(Debug)]
    enum AddressSeq<T> {
        Supernet(T),
        Hypernet(T),
    }
    impl<T: AsRef<str>> AddressSeq<T> {
        pub fn as_str(&self) -> &str {
            match self {
                AddressSeq::Supernet(s) => s.as_ref(),
                AddressSeq::Hypernet(s) => s.as_ref(),
            }
        }

        pub fn process_pattern<P>(&self) -> Vec<AddressSeq<P>>
        where
            P: Pattern,
            [(); P::SIZE]:,
        {
            let build = |f: fn(P) -> AddressSeq<P>| -> Vec<AddressSeq<P>> {
                let chars: Vec<_> = self.as_str().chars().collect();
                chars
                    .windows(P::SIZE)
                    .filter_map(|cs: &[char]| P::from_chars(cs.try_into().unwrap()).map(f))
                    .collect()
            };

            match self {
                AddressSeq::Supernet(_) => build(AddressSeq::Supernet),
                AddressSeq::Hypernet(_) => build(AddressSeq::Hypernet),
            }
        }
    }

    pub trait AddressStatus {}

    #[derive(Debug)]
    pub struct Raw {
        sequences: Vec<AddressSeq<String>>,
    }
    impl AddressStatus for Raw {}

    trait Pattern: Sized {
        const SIZE: usize;

        fn from_chars(chars: [char; Self::SIZE]) -> Option<Self>;
    }

    #[derive(Debug)]
    struct AbbaPattern {
        _chars: [char; 4],
    }
    impl Pattern for AbbaPattern {
        const SIZE: usize = 4;

        fn from_chars(chars: [char; Self::SIZE]) -> Option<Self> {
            (chars[0] != chars[1] && chars[0] == chars[3] && chars[1] == chars[2])
                .then(|| Self { _chars: chars })
        }
    }

    #[derive(Debug, PartialEq, Eq, Hash)]
    struct AbaPattern {
        chars: [char; 3],
    }
    impl Pattern for AbaPattern {
        const SIZE: usize = 3;

        fn from_chars(chars: [char; Self::SIZE]) -> Option<Self> {
            (chars[0] != chars[1] && chars[0] == chars[2]).then(|| Self { chars })
        }
    }
    impl Neg for &AbaPattern {
        type Output = AbaPattern;

        fn neg(self) -> Self::Output {
            let a = self.chars[0];
            let b = self.chars[1];
            AbaPattern { chars: [b, a, b] }
        }
    }

    #[derive(Debug)]
    pub struct Processed {
        abba_sequences: Vec<AddressSeq<AbbaPattern>>,
        aba_sequences: Vec<AddressSeq<AbaPattern>>,
    }
    impl AddressStatus for Processed {}

    #[derive(Debug)]
    struct IpAddress<S: AddressStatus> {
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
        pub fn process(self) -> AocResult<IpAddress<Processed>> {
            fn build<P>(addr: &IpAddress<Raw>) -> Vec<AddressSeq<P>>
            where
                P: Pattern,
                [(); P::SIZE]:,
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

    pub struct IpAddresses {
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
        pub fn num_support_tls(&self) -> u64 {
            self.addresses.iter().filter_count(|a| a.supports_tls())
        }

        pub fn num_support_ssl(&self) -> u64 {
            self.addresses.iter().filter_count(|a| a.supports_ssl())
        }
    }

    /* pub struct IpAddress(String);
    impl<'a> FromStr for IpAddress {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self(s.trim().into()))
        }
    }
    impl IpAddress {
        fn is_abba(buffer: &CircularBuffer<4, char>) -> bool {
            buffer.get(0) != buffer.get(1)
                && buffer.get(0) == buffer.get(3)
                && buffer.get(1) == buffer.get(2)
        }

        pub fn supports_tls(&self) -> AocResult<bool> {
            let mut buffer = CircularBuffer::new();
            let mut outside_abba = false;
            let mut in_hypernet = false;

            for c in self.0.chars() {
                // Add the char to the buffer
                buffer.push_back(c);

                // Do we have an ABBA?
                if buffer.len() == 4 {
                    if Self::is_abba(&buffer) {
                        if in_hypernet {
                            return Ok(false);
                        }
                        outside_abba = true;
                    }
                }

                // Handle the hypernet sequences
                if c == '[' {
                    (!in_hypernet).ok_or(AocError::Process("Nested hypernet sequences".into()))?;
                    in_hypernet = true;
                } else if c == ']' {
                    in_hypernet.ok_or(AocError::Process(
                        "Unmatched hypernet sqeuence closer ']'".into(),
                    ))?;
                    in_hypernet = false;
                }
            }

            Ok(outside_abba)
        }
    } */
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 7,
    name: "Internet Protocol Version 7",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let addresses = IpAddresses::from_str(input.expect_text()?)?;

            // Process
            Ok(addresses.num_support_tls().into())
        },
        // Part two
        |input| {
            // Generation
            let addresses = IpAddresses::from_str(input.expect_text()?)?;

            // Process
            Ok(addresses.num_support_ssl().into())
        },
    ],
};
