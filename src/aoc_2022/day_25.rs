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
        actual_answers = unsigned![123];
    }
}

/// Contains solution implementation items.
mod solution {
    use aoc::parse::trim;
    use derive_more::{Add, From, Sub};
    use nom::{branch::alt, bytes::complete::tag, combinator::map, multi::many1};

    use super::*;

    #[derive(Clone, Copy)]
    enum SnafuDigit {
        Zero,
        One,
        Two,
        DoubleMinus,
        Minus,
    }
    impl Parsable<'_> for SnafuDigit {
        fn parser(input: &'_ str) -> NomParseResult<&str, Self> {
            alt((
                map(tag("0"), |_| Self::Zero),
                map(tag("1"), |_| Self::One),
                map(tag("2"), |_| Self::Two),
                map(tag("="), |_| Self::DoubleMinus),
                map(tag("-"), |_| Self::Minus),
            ))(input)
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

        pub fn is_negative_digit(&self) -> bool {
            match self {
                Self::DoubleMinus | Self::Minus => true,
                _ => false,
            }
        }
    }

    #[derive(Debug, Add, Sub, From)]
    pub struct SnafuNumber(i64);
    impl Parsable<'_> for SnafuNumber {
        fn parser(input: &'_ str) -> NomParseResult<&str, Self> {
            // TODO: Verify that this works for negative SNAFU numbers.
            map(trim(false, many1(SnafuDigit::parser)), |mut digs| {
                digs.reverse();
                Self(digs.into_iter().enumerate().map(|(p, d)| d.value(p)).sum())
            })(input)
        }
    }
    impl std::fmt::Display for SnafuNumber {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut value = self.0;
            let mut digits = Vec::new();
            loop {
                // TODO: Need to get this working with negative numbers too.
                let digit = SnafuDigit::try_from(value.rem_euclid(5)).unwrap();
                println!(
                    "{} step {}: {value} mod 5 {}",
                    self.0,
                    digits.len(),
                    value.rem_euclid(5)
                );
                value /= 5;
                println!("{} step {}: {value} div 5 {}", self.0, digits.len(), value);
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
            let x = SnafuNumber::from_str("1121-1110-1=0")?;
            println!("TODO: {x:?}");

            let x = SnafuNumber::from(-13);
            println!("TODO: {x}");

            // Process
            Ok(0u64.into())
        },
    ],
};
