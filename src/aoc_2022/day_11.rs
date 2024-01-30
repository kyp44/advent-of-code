use aoc::prelude::*;
use itertools::Itertools;
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
            answers = unsigned![10605];
        }
        actual_answers = unsigned![58794];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
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

    #[derive(Debug, Clone, Copy)]
    enum Operator {
        Add,
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
        pub fn operator_fn(&self) -> fn(u32, u32) -> u32 {
            match self {
                Operator::Add => u32::add,
                Operator::Multiply => u32::mul,
            }
        }
    }

    #[derive(Debug, Clone)]
    enum Operand {
        Old,
        Number(u32),
    }
    impl Parsable<'_> for Operand {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            alt((
                map(tag("old"), |_| Self::Old),
                map(nom::character::complete::u32, Self::Number),
            ))(input)
        }
    }
    impl Operand {
        pub fn value(&self, old: u32) -> u32 {
            match self {
                Operand::Old => old,
                Operand::Number(n) => *n,
            }
        }
    }

    #[derive(Debug, Clone)]
    struct Operation {
        operands: [Operand; 2],
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
        pub fn evaluate(&self, old: u32) -> u32 {
            self.operation.operator_fn()(self.operands[0].value(old), self.operands[1].value(old))
        }
    }

    #[derive(Debug, Clone)]
    struct Test {
        div_by: u32,
        if_true: u8,
        if_false: u8,
    }
    impl Parsable<'_> for Test {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                tuple((
                    trim(
                        true,
                        preceded(tag("divisible by "), nom::character::complete::u32),
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
        pub fn evaluate(&self, worry_level: u32) -> u8 {
            if worry_level % self.div_by == 0 {
                self.if_true
            } else {
                self.if_false
            }
        }
    }

    struct Thrown {
        to_monkey: u8,
        item: u32,
    }

    #[derive(Debug, Clone)]
    struct Monkey {
        number: u8,
        items: Vec<u32>,
        operation: Operation,
        test: Test,
        inspected_items: u64,
    }
    impl Parsable<'_> for Monkey {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                tuple((
                    delimited(tag("Monkey "), nom::character::complete::u8, tag(":")),
                    preceded(
                        trim(true, tag("Starting items:")),
                        separated_list1(tag(", "), nom::character::complete::u32),
                    ),
                    preceded(trim(true, tag("Operation:")), Operation::parser),
                    preceded(trim(true, tag("Test:")), Test::parser),
                )),
                |(number, items, operation, test)| Self {
                    number,
                    items,
                    operation,
                    test,
                    inspected_items: 0,
                },
            )(input)
        }
    }
    impl std::fmt::Display for Monkey {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Monkey {}: {:?}", self.number, self.items)
        }
    }
    impl Monkey {
        pub fn take_turn(&mut self) -> Vec<Thrown> {
            self.items
                .drain(..)
                .map(|mut item| {
                    item = self.operation.evaluate(item);

                    self.inspected_items += 1;
                    item = item / 3;

                    Thrown {
                        to_monkey: self.test.evaluate(item),
                        item,
                    }
                })
                .collect()
        }
    }

    #[derive(Clone, Debug)]
    pub struct Monkeys {
        monkeys: IndexMap<u8, Monkey>,
    }
    impl FromStr for Monkeys {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let monkeys = Monkey::gather(s.split("\n\n"))?;

            Ok(Self {
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
        fn receive(&mut self, thrown: Thrown) -> AocResult<()> {
            match self.monkeys.get_mut(&thrown.to_monkey) {
                Some(monkey) => {
                    monkey.items.push(thrown.item);
                    Ok(())
                }
                None => Err(AocError::Process(
                    format!("Monkey {} does not exist!", thrown.to_monkey).into(),
                )),
            }
        }

        pub fn inspected_items(&self) -> impl Iterator<Item = u64> + '_ {
            self.monkeys.values().map(|m| m.inspected_items)
        }
    }
    impl Iterator for Monkeys {
        type Item = Monkeys;

        fn next(&mut self) -> Option<Self::Item> {
            let nums = self.monkeys.keys().copied().collect_vec();
            for num in nums {
                let throws = self.monkeys[&num].take_turn();
                for thrown in throws {
                    self.receive(thrown).unwrap();
                }
            }

            Some(self.clone())
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
            let monkeys = input
                .expect_data::<Monkeys>()?
                .clone()
                .skip(19)
                .next()
                .unwrap();

            let mut inspected_items = monkeys.inspected_items().collect_vec();
            inspected_items.sort_unstable();
            inspected_items.reverse();
            Ok((inspected_items[0] * inspected_items[1]).into())
        },
    ],
};
