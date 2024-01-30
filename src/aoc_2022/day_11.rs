use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0

Monkey 2:
  Starting items: 79, 60, 97
  Operation: new = old * old
  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3

Monkey 3:
  Starting items: 74
  Operation: new = old + 3
  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1";
            answers = unsigned![10605, 2713310158];
        }
        actual_answers = unsigned![58794, 20151213744];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use gat_lending_iterator::LendingIterator;
    use indexmap::IndexMap;
    use itertools::Itertools;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::map,
        multi::separated_list1,
        sequence::{delimited, preceded, tuple},
    };
    use std::ops::{Add, Mul};

    /// An arithmetic operator in an [`Operation`].
    #[derive(Debug, Clone, Copy)]
    enum Operator {
        /// Addition.
        Add,
        /// Multiplication.
        Multiply,
    }
    impl Parsable<'_> for Operator {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            trim(
                false,
                alt((
                    map(tag("+"), |_| Self::Add),
                    map(tag("*"), |_| Self::Multiply),
                )),
            )(input)
        }
    }
    impl Operator {
        /// Returns the operator function for this operator.
        pub fn operator_fn(&self) -> fn(u64, u64) -> u64 {
            match self {
                Operator::Add => u64::add,
                Operator::Multiply => u64::mul,
            }
        }
    }

    /// An operand used in an [`Operation`].
    #[derive(Debug, Clone)]
    enum Operand {
        /// Stand-in for the old worry level.
        Old,
        /// A number literal.
        Number(u64),
    }
    impl Parsable<'_> for Operand {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            alt((
                map(tag("old"), |_| Self::Old),
                map(nom::character::complete::u64, Self::Number),
            ))(input)
        }
    }
    impl Operand {
        /// Returns the actual value of operand given the `old` worry level.
        pub fn value(&self, old: u64) -> u64 {
            match self {
                Operand::Old => old,
                Operand::Number(n) => *n,
            }
        }
    }

    /// A binary arithmetic operation to apply during inspection to calculate a new worry level.
    #[derive(Debug, Clone)]
    struct Operation {
        /// The two operands involved.
        operands: [Operand; 2],
        /// The binary operator to combine the two operands.
        operation: Operator,
    }
    impl Parsable<'_> for Operation {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                preceded(
                    preceded(tag("new"), trim(false, tag("="))),
                    tuple((Operand::parser, Operator::parser, Operand::parser)),
                ),
                |(a, op, b)| Self {
                    operands: [a, b],
                    operation: op,
                },
            )(input)
        }
    }
    impl Operation {
        /// Evaluates the operation given the `old` worry level, returning the resulting new worry level.
        pub fn evaluate(&self, old: u64) -> u64 {
            self.operation.operator_fn()(self.operands[0].value(old), self.operands[1].value(old))
        }
    }

    /// The test a monkey does in order to determine to which monkey to throw the item after inspection.
    #[derive(Debug, Clone)]
    struct Test {
        /// Value by which the worry level must be divisible to pass the test.
        div_by: u64,
        /// Monkey number to which to throw the item if the test passes.
        if_true: u8,
        /// Monkey number to which to throw the item if the test fails.
        if_false: u8,
    }
    impl Parsable<'_> for Test {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                tuple((
                    trim(
                        true,
                        preceded(tag("divisible by "), nom::character::complete::u64),
                    ),
                    trim(
                        true,
                        preceded(
                            tag("If true: throw to monkey "),
                            nom::character::complete::u8,
                        ),
                    ),
                    trim(
                        true,
                        preceded(
                            tag("If false: throw to monkey "),
                            nom::character::complete::u8,
                        ),
                    ),
                )),
                |(div_by, if_true, if_false)| Self {
                    div_by,
                    if_true,
                    if_false,
                },
            )(input)
        }
    }
    impl Test {
        /// Evaluates the test given the `worry_level`, returning the monkey number to which to throw the item.
        pub fn evaluate(&self, worry_level: u64) -> u8 {
            if worry_level % self.div_by == 0 {
                self.if_true
            } else {
                self.if_false
            }
        }
    }

    /// Represents the throwing of a particular item to a particular monkey.

    struct Thrown {
        /// Monkey number to which to throw the item.
        to_monkey: u8,
        /// The worry level for the item.
        item_worry_level: u64,
    }

    /// A monkey just monkeying around.
    #[derive(Debug, Clone)]
    struct Monkey {
        /// The monkey number for this monkey.
        number: u8,
        /// Item worry levels in the order in which to be inspected.
        item_worry_levels: Vec<u64>,
        /// Operation to which to apply to the worry level during inspection to get a new worry level.
        operation: Operation,
        /// A test to determine to which other monkey to throw the item after inspection.
        test: Test,
        /// The number of items inspected so far.
        inspected_items: u64,
    }
    impl Parsable<'_> for Monkey {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                tuple((
                    delimited(tag("Monkey "), nom::character::complete::u8, tag(":")),
                    preceded(
                        trim(true, tag("Starting items:")),
                        separated_list1(tag(", "), nom::character::complete::u64),
                    ),
                    preceded(trim(true, tag("Operation:")), Operation::parser),
                    preceded(trim(true, tag("Test:")), Test::parser),
                )),
                |(number, item_worry_levels, operation, test)| Self {
                    number,
                    item_worry_levels,
                    operation,
                    test,
                    inspected_items: 0,
                },
            )(input)
        }
    }
    impl std::fmt::Display for Monkey {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Monkey {}: {:?}", self.number, self.item_worry_levels)
        }
    }
    impl Monkey {
        /// Takes the next turn for this monkey.
        ///
        /// If `modulo` is passed, the operation will be restricted to that modulo system (part two),
        /// otherwise the new worry level will be divided by three before the test (part one).
        /// Refer to the notes for more detail about this.
        ///
        /// Returns [`Thrown`] objects, indicating to which monkey each item should be thrown.
        pub fn take_turn(&mut self, modulo: Option<u64>) -> Vec<Thrown> {
            self.item_worry_levels
                .drain(..)
                .map(|mut item| {
                    item = self.operation.evaluate(item);
                    self.inspected_items += 1;

                    match modulo {
                        Some(m) => item %= m,
                        None => item /= 3,
                    }

                    Thrown {
                        to_monkey: self.test.evaluate(item),
                        item_worry_level: item,
                    }
                })
                .collect()
        }
    }

    /// A band of monkeys playing Keep Away and throwing your items to each other.
    #[derive(Clone, Debug)]
    pub struct Monkeys {
        /// Optional modulo system in which to perform the worry level arithmetic, see the notes.
        modulo: Option<u64>,
        /// Map of the monkey numbers to the [`Monkey`] object.
        monkeys: IndexMap<u8, Monkey>,
    }
    impl FromStr for Monkeys {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let monkeys = Monkey::gather(s.split("\n\n"))?;

            Ok(Self {
                modulo: None,
                monkeys: monkeys.into_iter().map(|m| (m.number, m)).collect(),
            })
        }
    }
    impl std::fmt::Display for Monkeys {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for monkey in self.monkeys.values() {
                writeln!(f, "{monkey}")?;
            }
            Ok(())
        }
    }
    impl Monkeys {
        /// Applies the throwing of an item.
        ///
        /// This can fail if the monkey being thrown to does not exist.
        fn receive(&mut self, thrown: Thrown) -> AocResult<()> {
            match self.monkeys.get_mut(&thrown.to_monkey) {
                Some(monkey) => {
                    monkey.item_worry_levels.push(thrown.item_worry_level);
                    Ok(())
                }
                None => Err(AocError::Process(
                    format!("Monkey {} does not exist!", thrown.to_monkey).into(),
                )),
            }
        }

        /// Disables worry reduction for these monkeys and instead determines the correct
        /// modulo system in which to do the worry level arithmetic.
        ///
        /// Refer to the notes for more details about this.
        pub fn disable_worry_reduction(&mut self) {
            self.modulo = Some(
                self.monkeys
                    .values()
                    .map(|m| m.test.div_by)
                    .product::<u64>(),
            );
        }

        /// Has the monkeys take turns for some number of `rounds` and returns the the level
        /// of monkey business after the final turn.
        pub fn monkey_business(&mut self, rounds: usize) -> u64 {
            let _ = self.nth(rounds - 1).unwrap();

            let mut inspected_items = self
                .monkeys
                .values()
                .map(|m| m.inspected_items)
                .collect_vec();
            inspected_items.sort_unstable();
            inspected_items.reverse();
            inspected_items[0] * inspected_items[1]
        }
    }
    impl LendingIterator for Monkeys {
        type Item<'a> = &'a Monkeys
        where
            Self: 'a;

        fn next(&mut self) -> Option<Self::Item<'_>> {
            let nums = self.monkeys.keys().copied().collect_vec();
            for num in nums {
                let throws = self.monkeys[&num].take_turn(self.modulo);
                for thrown in throws {
                    self.receive(thrown).unwrap();
                }
            }

            Some(self)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 11,
    name: "Monkey in the Middle",
    preprocessor: Some(|input| Ok(Box::new(Monkeys::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Monkeys>()?
                .clone()
                .monkey_business(20)
                .into())
        },
        // Part two
        |input| {
            // Generate
            let mut monkeys = input.expect_data::<Monkeys>()?.clone();
            monkeys.disable_worry_reduction();

            // Process
            Ok(monkeys.monkey_business(10000).into())
        },
    ],
};
