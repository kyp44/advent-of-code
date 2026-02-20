use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "1=-0-2
12111
2=0=
21
2=01
111
20012
112
1=-1=
1-12
12
1=
122";
            answers = string!["2=-1=0"];
        }
        actual_answers = string!["2----0=--1122=0=0021"];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::iter::Sum;

    use aoc::parse::trim;
    use derive_more::{Add, From, Sub};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{all_consuming, map},
        multi::many1,
    };

    use super::*;

    /// A single digit in a SNAFU number, which can be parsed from text
    /// input.
    #[derive(Clone, Copy)]
    enum SnafuDigit {
        /// Zero (`0`).
        Zero,
        /// One (`1`).
        One,
        /// Two (`2`).
        Two,
        /// Double minus (`=`).
        DoubleMinus,
        /// Minus (`-`).
        Minus,
    }
    impl Parsable for SnafuDigit {
        fn parser(input: &'_ str) -> NomParseResult<&str, Self> {
            alt((
                map(tag("0"), |_| Self::Zero),
                map(tag("1"), |_| Self::One),
                map(tag("2"), |_| Self::Two),
                map(tag("="), |_| Self::DoubleMinus),
                map(tag("-"), |_| Self::Minus),
            ))
            .parse(input)
        }
    }
    impl TryFrom<i64> for SnafuDigit {
        type Error = AocError;

        fn try_from(value: i64) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(Self::Zero),
                1 => Ok(Self::One),
                2 => Ok(Self::Two),
                3 => Ok(Self::DoubleMinus),
                4 => Ok(Self::Minus),
                _ => Err(AocError::Process(
                    format!("{} is not cannot be converted to a SNAFU digit", value).into(),
                )),
            }
        }
    }
    impl std::fmt::Display for SnafuDigit {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    SnafuDigit::Zero => '0',
                    SnafuDigit::One => '1',
                    SnafuDigit::Two => '2',
                    SnafuDigit::DoubleMinus => '=',
                    SnafuDigit::Minus => '-',
                }
            )
        }
    }
    impl SnafuDigit {
        /// Returns the value of the digit, given its `place` in the SNAFU
        /// number.
        ///
        /// A `place` of `0` is the rightmost (ones) digit while a `place` of
        /// `n-1` is the leftmost digit if the number has `n` digits.
        pub fn value(&self, place: usize) -> i64 {
            5i64.pow(place.try_into().unwrap())
                * match self {
                    SnafuDigit::Zero => 0,
                    SnafuDigit::One => 1,
                    SnafuDigit::Two => 2,
                    SnafuDigit::DoubleMinus => -2,
                    SnafuDigit::Minus => -1,
                }
        }

        /// Returns whether the digit has a negative value, that is, whether it
        /// is [`SnafuDigit::Minus`] or [`SnafuDigit::DoubleMinus`].
        pub fn is_negative_digit(&self) -> bool {
            matches!(self, Self::DoubleMinus | Self::Minus)
        }
    }

    /// A single SNAFU number, which can be parsed from text input.
    #[derive(Debug, Add, Sub, From)]
    pub struct SnafuNumber(i64);
    impl Parsable for SnafuNumber {
        fn parser(input: &'_ str) -> NomParseResult<&str, Self> {
            map(
                trim(
                    false,
                    all_consuming::<_, NomParseError, _>(many1(SnafuDigit::parser)),
                ),
                |mut digs| {
                    digs.reverse();
                    Self(digs.into_iter().enumerate().map(|(p, d)| d.value(p)).sum())
                },
            )
            .parse(input)
        }
    }
    impl std::fmt::Display for SnafuNumber {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut value = self.0;
            let mut digits = Vec::new();
            loop {
                let (d, m) = num_integer::div_mod_floor(value, 5);
                let digit = SnafuDigit::try_from(m).unwrap();

                value = d;

                if digit.is_negative_digit() {
                    // Need to increment the next digit
                    value += 1;
                }

                digits.push(digit);
                if value == 0 {
                    // Nothing left
                    break;
                }
            }
            digits.reverse();
            for d in digits {
                write!(f, "{d}")?;
            }
            Ok(())
        }
    }
    impl Sum for SnafuNumber {
        fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.reduce(|a, b| a + b).unwrap_or(0.into())
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 25,
    name: "Full of Hot Air",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let numbers = SnafuNumber::gather(input.expect_text()?.lines())?;

            // Process
            let sum: SnafuNumber = numbers.into_iter().sum();
            Ok(sum.to_string().into())
        },
    ],
};
