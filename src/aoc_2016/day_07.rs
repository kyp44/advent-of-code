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
            answers = unsigned![2];
        }
        actual_answers = unsigned![105];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use gat_lending_iterator::{LendingIterator, ToLendingIterator};
    use nom::{
        branch::alt, bytes::complete::tag, character::complete::alpha1, combinator::map,
        multi::many1, sequence::delimited,
    };
    use std::ops::Neg;

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

        pub fn process_abbas(&self) -> Vec<AddressSeq<AbbaSection>> {
            /* let abbas = self
                .as_str()
                .chars()
                .collect::<Vec<_>>()
                .windows(4)
                .filter_map(|cs: &[char]| AbbaSection::from_chars(cs.try_into().unwrap()));

            match self {
                AddressSeq::Supernet(_) => abbas.map(|a| AddressSeq::Supernet(a)).collect(),
                AddressSeq::Hypernet(_) => abbas.map(|a| AddressSeq::Hypernet(a)).collect(),
            } */
            let chars: Vec<_> = "What the fuck".chars().collect();
            let abbas: Vec<_> = chars
                .windows(4)
                .filter_map(|cs: &[char]| -> Option<AbbaSection> {
                    let a: [char; 4] = cs.try_into().unwrap();
                    AbbaSection::from_chars(a)
                })
                .collect();

            todo!()
        }
    }

    pub trait AddressStatus {}

    #[derive(Debug)]
    pub struct Raw {
        sequences: Vec<AddressSeq<String>>,
    }
    impl AddressStatus for Raw {}

    #[derive(Debug)]
    struct AbbaSection {
        chars: [char; 4],
    }
    impl AbbaSection {
        pub fn from_chars(chars: [char; 4]) -> Option<Self> {
            (chars[0] != chars[1] && chars[0] == chars[3] && chars[1] == chars[2])
                .then(|| Self { chars })
        }
    }

    #[derive(Debug)]
    pub struct Processed {
        abba_sequences: Vec<AddressSeq<AbbaSection>>,
        //aba_sequences: Vec<AddressSeq>,
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
        fn ascii_windows(s: &str, n: usize) -> impl Iterator<Item = &str> {
            (0..=s.len().saturating_sub(n)).map(move |i| &s[i..i + n])
        }

        pub fn process(self) -> AocResult<IpAddress<Processed>> {
            let mut abba_sequences = Vec::new();
            //let mut aba_sequences = Vec::new();

            for seq in self.status.sequences.into_iter() {
                seq.is_ascii().ok_or(AocError::Process(
                    "There is a sequence that is not ASCII".into(),
                ))?;

                let abbas =
                    Self::ascii_windows(seq.as_str(), 4).filter_map(|s| AbbaSection::from_str(s));

                match seq {
                    AddressSeq::Supernet(_) => {
                        abba_sequences.extend(abbas.map(|a| AddressSeq::Supernet(a)))
                    }
                    AddressSeq::Hypernet(_) => {
                        abba_sequences.extend(abbas.map(|a| AddressSeq::Hypernet(a)))
                    }
                }
            }

            Ok(IpAddress {
                status: Processed {
                    abba_sequences,
                    //aba_sequences,
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
        /* |input| {
            // Generation
            let addresses = input
                .expect_text()?
                .lines()
                .map(|s| IpAddress::from_str(s))
                .collect::<AocResult<Vec<_>>>()?;

            // Process
            let mut count = 0u64;
            for addr in addresses.iter() {
                if addr.supports_tls()? {
                    count += 1;
                }
            }

            Ok(count.into())
        }, */
    ],
};
