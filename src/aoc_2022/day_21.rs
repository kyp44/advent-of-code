use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "root: pppw + sjmn
dbpl: 5
cczh: sllz + lgvd
zczc: 2
ptdq: humn - dvpt
dvpt: 3
lfqf: 4
humn: 5
ljgn: 2
sjmn: drzm * dbpl
sllz: 4
pppw: cczh / lfqf
lgvd: ljgn * ptdq
drzm: hmdt - zczc
hmdt: 32";
            answers = unsigned![123];
        }
        actual_answers = unsigned![123];
    }
}

/// Contains solution implementation items.
mod solution {
    use aoc::parse::{separated, trim};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::alpha1,
        combinator::map,
        sequence::{separated_pair, tuple},
    };

    use super::*;

    type Num = u32;

    enum Operation {
        Add,
        Subtract,
        Multiply,
        Divide,
    }
    impl Operation {
        pub fn apply(&self, a: Num, b: Num) -> AocResult<u32> {
            Ok(match self {
                Operation::Add => a + b,
                Operation::Subtract => {
                    if a <= b {
                        a - b
                    } else {
                        return Err(AocError::Process(
                            format!("subtracting {b} from {a} would be negative").into(),
                        ));
                    }
                }
                Operation::Multiply => a * b,
                Operation::Divide => {
                    if a % b == 0 {
                        a / b
                    } else {
                        return Err(AocError::Process(
                            format!("{a} is not divisible by {b}").into(),
                        ));
                    }
                }
            })
        }
    }
    impl Parsable<'_> for Operation {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            alt((
                map(trim(false, tag("+")), |_| Self::Add),
                map(trim(false, tag("-")), |_| Self::Subtract),
                map(trim(false, tag("*")), |_| Self::Multiply),
                map(trim(false, tag("/")), |_| Self::Divide),
            ))(input)
        }
    }

    enum MonkeyAction<S> {
        Yell(Num),
        Math { operation: Operation, a: S, b: S },
    }
    impl<'a> Parsable<'a> for MonkeyAction<&'a str> {
        fn parser(input: &'a str) -> NomParseResult<&str, Self> {
            alt((
                map(nom::character::complete::u32, |n| Self::Yell(n)),
                map(
                    tuple((alpha1, Operation::parser, alpha1)),
                    |(a, operation, b)| Self::Math { operation, a, b },
                ),
            ))(input)
        }
    }

    struct MonkeyParse<'a> {
        name: &'a str,
        action: MonkeyAction<&'a str>,
    }
    impl<'a> Parsable<'a> for MonkeyParse<'a> {
        fn parser(input: &'a str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(alpha1, trim(false, tag(":")), MonkeyAction::parser),
                |(name, action)| Self { name, action },
            )(input)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 21,
    name: "Monkey Math",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation

            // Process
            Ok(0u64.into())
        },
    ],
};
