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
            answers = unsigned![152, 301];
        }
        // NOTE: This example tests things that the given example does not but the
        // actual input does exercise for part 2. This was taken from here:
        // https://www.reddit.com/r/adventofcode/comments/zrtw6y/2022_day_21_part_2_another_example/
        example {
            input = "nice: 6
earl: dark / sour
chip: tofu * rice
pure: 3
root: tree * chip
tofu: 3
dams: 3
grey: 5377482938105
corn: 31
cane: earl - cake
ruby: 7
humn: 69
cake: nice * ruby
bars: 77
rice: dams * pure
dark: 53900
lone: 5377482938110
milk: humn + corn
sour: bars * pies
tree: bell - cane
pink: lone - grey
bell: 55
pies: milk / pink";
            answers = unsigned![1674, 19];
        }
        actual_answers = unsigned![54703080378102, 3952673930912];
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
    use std::{borrow::Cow, collections::HashMap};

    /// The numeric type used for all monkey arithmetic.
    type Num = i64;

    /// An arithmetic operation, which can be parsed from text input.
    #[derive(Debug, Clone, Copy)]
    pub enum Operation {
        /// Addition.
        Add,
        /// Subtraction.
        Subtract,
        /// Multiplication.
        Multiply,
        /// Division (integer division only).
        Divide,
        /// Equality for solving an equation (part two only).
        Equals,
    }
    impl std::ops::Neg for Operation {
        type Output = Self;

        fn neg(self) -> Self::Output {
            match self {
                Operation::Add => Operation::Subtract,
                Operation::Subtract => Operation::Add,
                Operation::Multiply => Operation::Divide,
                Operation::Divide => Operation::Multiply,
                Operation::Equals => Operation::Equals,
            }
        }
    }
    impl Operation {
        /// Applies the operation for left and right operands `a` and `b`,
        /// respectively.
        ///
        /// This will return an error for division if `a` is not divisible
        /// by `b`, or if the operation is [`Operation::Equals`], which
        /// cannot be applied.
        pub fn apply(&self, a: Num, b: Num) -> AocResult<Num> {
            Ok(match self {
                Operation::Add => a + b,
                Operation::Subtract => a - b,
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
                Operation::Equals => {
                    return Err(AocError::Process(
                        "Cannot apply the equality operation".into(),
                    ))
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

    /// An action that a monkey can take, which can be parsed from text input.
    #[derive(Debug, Clone)]
    pub enum MonkeyAction<S> {
        /// Yell an explicit number.
        Yell(Num),
        /// Yell a number that must be determined (for you, only in part two).
        Unknown,
        /// Performs arithmetic based on what other monkeys yell.
        Arithmetic {
            /// The arithmetic operation.
            operation: Operation,
            /// The name of the monkey who will yell the left operand.
            a: S,
            /// The name of the monkey who will yell the right operand.
            b: S,
        },
    }
    impl<'a> Parsable<'a> for MonkeyAction<&'a str> {
        fn parser(input: &'a str) -> NomParseResult<&str, Self> {
            alt((
                map(nom::character::complete::i64, MonkeyAction::Yell),
                map(
                    tuple((alpha1, Operation::parser, alpha1)),
                    |(a, operation, b)| Self::Arithmetic { operation, a, b },
                ),
            ))(input)
        }
    }
    impl From<MonkeyAction<&str>> for MonkeyAction<String> {
        fn from(value: MonkeyAction<&str>) -> Self {
            match value {
                MonkeyAction::Yell(n) => MonkeyAction::Yell(n),
                MonkeyAction::Unknown => MonkeyAction::Unknown,
                MonkeyAction::Arithmetic { operation, a, b } => MonkeyAction::Arithmetic {
                    operation,
                    a: a.to_string(),
                    b: b.to_string(),
                },
            }
        }
    }

    /// A monkey and its action, which can be parsed from text input.
    struct MonkeyParse<'a> {
        /// The name of the monkey.
        name: &'a str,
        /// The action that the monkey takes in order to yell its number.
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

    /// A map from the monkey name to its action.
    type MonkeyMap = HashMap<String, MonkeyAction<String>>;

    /// The position of a single operand in an arithmetic operation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum OperandPosition {
        /// The left/first operand.
        Left,
        /// The right/second operand.
        Right,
    }

    /// An arithmetic operation in which one operand is an [`Expression`]
    /// containing an unknown variable and the other is a known number.
    struct ExpressionOperation {
        /// The arithmetic operation.
        operation: Operation,
        /// The operand expression containing the unknown variable.
        unknown: Box<Expression>,
        /// The known, numeric operand.
        known: Num,
        /// The position of the expression containing the unknown variable.
        unknown_operand: OperandPosition,
    }

    /// A recursive arithmetic expression.
    #[derive(Debug)]
    enum Expression {
        /// Just a numeric value.
        Number(Num),
        /// An unknown variable.
        Unknown,
        /// An arithmetic operation with operands that are themselves
        /// expressions.
        Arithmetic {
            /// The arithmetic operation.
            operation: Operation,
            /// The left operand.
            a: Box<Expression>,
            /// The right operand.
            b: Box<Expression>,
        },
    }
    impl Expression {
        /// Creates the recursive `root` expression from a [`MonkeyMap`]
        /// for a particular [`Part`].
        pub fn from_monkeys<P: Part>(monkeys: &MonkeyMap) -> AocResult<Self> {
            /// This is a recursive internal function of [`Expression::from_monkeys`].
            fn convert_rec<P: Part>(monkeys: &MonkeyMap, name: &str) -> AocResult<Expression> {
                let action = P::get_monkey_action(monkeys, name)?;

                Ok(match action.as_ref() {
                    MonkeyAction::Yell(n) => Expression::Number(*n),
                    MonkeyAction::Unknown => Expression::Unknown,
                    MonkeyAction::Arithmetic { operation, a, b } => Expression::Arithmetic {
                        operation: *operation,
                        a: Box::new(convert_rec::<P>(monkeys, a)?),
                        b: Box::new(convert_rec::<P>(monkeys, b)?),
                    },
                })
            }

            convert_rec::<P>(monkeys, "root")
        }

        /// Attempts to recursively evaluate the expression to a single number.
        ///
        /// Returns the final number if this is possible, or [`None`] if the expression
        /// contains an [`Expression::Unknown`].
        /// An error can also be returned if something goes wrong.
        pub fn try_to_reduce(&self) -> AocResult<Option<Num>> {
            Ok(match self {
                Expression::Number(n) => Some(*n),
                Expression::Unknown => None,
                Expression::Arithmetic { operation, a, b } => {
                    if let Some(a) = a.try_to_reduce()?
                        && let Some(b) = b.try_to_reduce()?
                    {
                        Some(operation.apply(a, b)?)
                    } else {
                        None
                    }
                }
            })
        }

        /// Solves the equation, assuming this expression is an arithmetic
        /// [`Operation::Equals`] operation that contains a single unknown.
        ///
        /// Returns an error of this is not an equality operation, or if exactly
        /// one side is not an expression containing an unknown.
        /// Otherwise returns the necessary value of the unknown in order for
        /// the equality to be true.
        pub fn solve_equation(self) -> AocResult<Num> {
            let eo = self.expression_operation()?;

            match eo.operation {
                Operation::Equals => eo.unknown.solve_expression(eo.known),
                _ => Err(AocError::Process(
                    "The expression is not an equation".into(),
                )),
            }
        }

        /// Solves an expression containing an unknown variable when set equal
        /// to the `equals` number.
        ///
        /// Returns the required value of the unknown, or an error if the
        /// required conditions are not met.
        fn solve_expression(self, equals: Num) -> AocResult<Num> {
            if let Expression::Unknown = self {
                return Ok(equals);
            }

            let eo = self.expression_operation()?;

            match eo.operation {
                Operation::Equals => Err(AocError::Process(
                    "The expression cannot be an equation".into(),
                )),
                Operation::Subtract | Operation::Divide
                    if eo.unknown_operand == OperandPosition::Right =>
                {
                    Ok(eo.operation.apply(eo.known, equals)?)
                }
                _ => Ok((-eo.operation).apply(equals, eo.known)?),
            }
            .and_then(|n| eo.unknown.solve_expression(n))
        }

        /// Returns an [`ExpressionOperation`] for a arithmetic operation expression
        /// in which exactly one operand contains an unknown variable.
        ///
        /// An error is returned if the required conditions are not met.
        fn expression_operation(self) -> AocResult<ExpressionOperation> {
            match self {
                Expression::Arithmetic {
                    operation,
                    a: ae,
                    b: be,
                } => {
                    let a = ae.try_to_reduce()?;
                    let b = be.try_to_reduce()?;

                    match a {
                        Some(a) => match b {
                            Some(_) => Err("Expression contains no unknown side"),
                            None => Ok(ExpressionOperation {
                                operation,
                                unknown: be,
                                known: a,
                                unknown_operand: OperandPosition::Right,
                            }),
                        },
                        None => match b {
                            Some(b) => Ok(ExpressionOperation {
                                operation,
                                unknown: ae,
                                known: b,
                                unknown_operand: OperandPosition::Left,
                            }),
                            None => Err("Expression contains no numerical side"),
                        },
                    }
                }
                _ => Err("The expression is not an arithmetic operation"),
            }
            .map_err(|s| AocError::Process(s.into()))
        }
    }

    /// Behavior specific to each part of the problem.
    pub trait Part {
        /// Looks up the action, given the [`MonkeyMap`], for a monkey with a
        /// given `name` and returns it.
        ///
        /// This allows for the injection of alternative actions for special monkeys.
        fn get_monkey_action<'a>(
            monkeys: &'a MonkeyMap,
            name: &str,
        ) -> AocResult<Cow<'a, MonkeyAction<String>>>;

        /// Solves the given `riddle` for this part of the problem and returns
        /// the resulting number.
        fn solve(riddle: &Riddle) -> AocResult<Num>;
    }

    /// Solution for part one.
    ///
    /// Here the `root` expression is simply evaluated recursively to arrive
    /// at the final number yelled by the `root` monkey.
    pub struct PartOne {}
    impl Part for PartOne {
        fn solve(riddle: &Riddle) -> AocResult<Num> {
            let expression = Expression::from_monkeys::<Self>(&riddle.monkeys)?;

            expression.try_to_reduce()?.ok_or(AocError::Process(
                "Could not reduce the main expression".into(),
            ))
        }

        fn get_monkey_action<'a>(
            monkeys: &'a MonkeyMap,
            name: &str,
        ) -> AocResult<Cow<'a, MonkeyAction<String>>> {
            monkeys
                .get(name)
                .ok_or_else(|| Riddle::monkey_not_found(name))
                .map(Cow::Borrowed)
        }
    }

    /// Solution for part two.
    ///
    /// Here the `root` is an equality relation and the `humn` is you.
    /// The returned number is then the number that you must yell in order
    /// to make the equality true.
    pub struct PartTwo {}
    impl Part for PartTwo {
        fn get_monkey_action<'a>(
            monkeys: &'a MonkeyMap,
            name: &str,
        ) -> AocResult<Cow<'a, MonkeyAction<String>>> {
            let action = monkeys
                .get(name)
                .ok_or_else(|| Riddle::monkey_not_found(name))?;

            Ok(match name {
                "humn" => Cow::Owned(MonkeyAction::Unknown),
                "root" => match action {
                    MonkeyAction::Arithmetic { operation: _, a, b } => {
                        Cow::Owned(MonkeyAction::Arithmetic {
                            operation: Operation::Equals,
                            a: a.clone(),
                            b: b.clone(),
                        })
                    }
                    _ => {
                        return Err(AocError::Process(
                            "'root' is not an arithmetic operation".into(),
                        ))
                    }
                },
                _ => Cow::Borrowed(action),
            })
        }

        fn solve(riddle: &Riddle) -> AocResult<Num> {
            Expression::from_monkeys::<Self>(&riddle.monkeys)?.solve_equation()
        }
    }

    /// The riddle consisting of monkeys and their actions, which can be
    /// parsed from text input.
    #[derive(Debug)]
    pub struct Riddle {
        /// The map of monkey names to their actions.
        monkeys: MonkeyMap,
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
        /// Returns an error with an appropriate message any time the monkey
        /// with `name` cannot be found in a map.
        fn monkey_not_found(name: &str) -> AocError {
            AocError::Process(format!("Monkey '{name}' not found").into())
        }

        /// Solves the problem for a particular [`Part`].
        pub fn solve<P: Part>(&self) -> AocResult<u64> {
            P::solve(self)?
                .try_into()
                .map_err(|_| AocError::Process("The answer is negative".into()))
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 21,
    name: "Monkey Math",
    preprocessor: Some(|input| Ok(Box::new(Riddle::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<Riddle>()?.solve::<PartOne>()?.into())
        },
        // Part two
        |input| {
            // Process
            Ok(input.expect_data::<Riddle>()?.solve::<PartTwo>()?.into())
        },
    ],
};
