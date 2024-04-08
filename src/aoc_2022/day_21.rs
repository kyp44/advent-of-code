use aoc::prelude::*;
use std::str::FromStr;

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
            answers = unsigned![152];
        }
        actual_answers = unsigned![54703080378102];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::alpha1,
        combinator::map,
        sequence::{separated_pair, tuple},
    };
    use std::collections::HashMap;

    type Num = u64;

    #[derive(Debug)]
    enum Operation {
        Add,
        Subtract,
        Multiply,
        Divide,
    }
    impl Operation {
        pub fn apply(&self, a: Num, b: Num) -> AocResult<Num> {
            Ok(match self {
                Operation::Add => a + b,
                Operation::Subtract => {
                    if a >= b {
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

    #[derive(Debug)]
    enum MonkeyAction<S> {
        Yell(Num),
        Math { operation: Operation, a: S, b: S },
    }
    impl<'a> Parsable<'a> for MonkeyAction<&'a str> {
        fn parser(input: &'a str) -> NomParseResult<&str, Self> {
            alt((
                map(nom::character::complete::u64, Self::Yell),
                map(
                    tuple((alpha1, Operation::parser, alpha1)),
                    |(a, operation, b)| Self::Math { operation, a, b },
                ),
            ))(input)
        }
    }
    impl From<MonkeyAction<&str>> for MonkeyAction<String> {
        fn from(value: MonkeyAction<&str>) -> Self {
            match value {
                MonkeyAction::Yell(n) => MonkeyAction::Yell(n),
                MonkeyAction::Math { operation, a, b } => MonkeyAction::Math {
                    operation,
                    a: a.to_string(),
                    b: b.to_string(),
                },
            }
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

    #[derive(Debug)]
    pub struct Riddle {
        monkeys: HashMap<String, MonkeyAction<String>>,
    }
    impl FromStr for Riddle {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                monkeys: MonkeyParse::gather(s.lines())?
                    .into_iter()
                    .map(|m| (m.name.to_string(), m.action.into()))
                    .collect(),
            })
        }
    }
    impl Riddle {
        fn monkey_not_found(name: &str) -> String {
            format!("Monkey '{name}' not found")
        }

        pub fn solve(&self) -> AocResult<u64> {
            let mut values = HashMap::new();

            fn solve_rec<'a>(
                monkeys: &'a HashMap<String, MonkeyAction<String>>,
                values: &mut HashMap<&'a str, Num>,
                evaluate: &'a str,
            ) -> AocResult<Num> {
                if let Some(n) = values.get(evaluate) {
                    return Ok(*n);
                }

                let action = monkeys
                    .get(evaluate)
                    .ok_or_else(|| AocError::Process(Riddle::monkey_not_found(evaluate).into()))?;

                match action {
                    MonkeyAction::Yell(n) => {
                        let n = *n;
                        values.insert(evaluate, n);
                        Ok(n)
                    }
                    MonkeyAction::Math { operation, a, b } => operation.apply(
                        solve_rec(monkeys, values, a)?,
                        solve_rec(monkeys, values, b)?,
                    ),
                }
            }

            solve_rec(&self.monkeys, &mut values, "root")
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
            let riddle = Riddle::from_str(input.expect_input()?)?;

            // Process
            Ok(riddle.solve()?.into())
        },
    ],
};
