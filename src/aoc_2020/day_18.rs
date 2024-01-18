use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "1 + 2 * 3 + 4 * 5 + 6";
            answers = unsigned![71, 231];
        }
        example {
            input = "1 + (2 * 3) + (4 * (5 + 6))";
            answers = unsigned![51, 51];
        }
        example {
            input = "2 * 3 + (4 * 5)";
            answers = unsigned![26, 46];
        }
        example {
            input = "5 + (8 * 3 + 9 + 3 * 4 * 3)";
            answers = unsigned![437, 1445];
        }
        example {
            input = "5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))";
            answers = unsigned![12240, 669060];
        }
        example {
            input = "((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2";
            answers = unsigned![13632, 23340];
        }
        actual_answers = unsigned![464478013511, 85660197232452];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use itertools::process_results;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{all_consuming, map},
        multi::many1,
    };
    use std::cmp::Ordering;

    /// An operator that can appear in an expression.
    #[derive(Debug)]
    pub enum Operator {
        /// Addition operator.
        Add,
        /// Multiplication operator.
        Mul,
    }
    impl Operator {
        /// Evaluates the operation given the two operands.
        fn evaluate(&self, a: &u64, b: &u64) -> u64 {
            match self {
                Operator::Add => a + b,
                Operator::Mul => a * b,
            }
        }

        /// Compares the operator based on the precedence defined in the [`Part`].
        fn cmp(&self, other: &Operator, part: &dyn Part) -> Ordering {
            part.precedence(self).cmp(&part.precedence(other))
        }
    }

    /// Behavior specific to one particular part of the problem.
    pub trait Part {
        /// Converts an operator to its precedence number.
        ///
        /// Higher numbered operators are evaluated first.
        fn precedence(&self, op: &Operator) -> u8;
    }

    /// Behavior specific to part one.
    pub struct PartOne;
    impl Part for PartOne {
        fn precedence(&self, op: &Operator) -> u8 {
            match op {
                Operator::Add => 1,
                Operator::Mul => 1,
            }
        }
    }

    /// Behavior specific to part two.
    pub struct PartTwo;
    impl Part for PartTwo {
        fn precedence(&self, op: &Operator) -> u8 {
            match op {
                Operator::Add => 2,
                Operator::Mul => 1,
            }
        }
    }

    /// A parenthesis.
    #[derive(Debug)]
    enum Paren {
        /// Opening parenthesis.
        Start,
        /// Closing parenthesis.
        End,
    }

    /// An element of an expression.
    #[derive(Debug)]
    enum Element {
        /// A number literal.
        Number(u64),
        /// An operator.
        Operator(Operator),
        /// A parenthesis.
        Paren(Paren),
    }

    /// An arithmetic expression, which can be parsed from text input.
    #[derive(Debug)]
    struct Expression {
        /// Orginal expression.
        original: String,
        /// The list of parsed elements.
        elements: Vec<Element>,
    }
    impl Parseable<'_> for Expression {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            all_consuming(map(
                many1(alt((
                    map(trim(false, nom::character::complete::u64), Element::Number),
                    map(trim(false, tag("+")), |_| Element::Operator(Operator::Add)),
                    map(trim(false, tag("*")), |_| Element::Operator(Operator::Mul)),
                    map(trim(false, tag("(")), |_| Element::Paren(Paren::Start)),
                    map(trim(false, tag(")")), |_| Element::Paren(Paren::End)),
                ))),
                |elements| Expression {
                    original: input.to_string(),
                    elements,
                },
            ))(input.trim())
        }
    }
    impl Expression {
        /// Verifies that an expression is valid and does not contain things like
        /// two operands in a row or mismatched parenthesis.
        fn is_valid(&self) -> bool {
            let mut depth: i32 = 0;
            let mut iter = self.elements.iter();
            let mut expect_num = true;
            loop {
                if expect_num {
                    // Expecting a number or sub-expression
                    match iter.next() {
                        None => return false,
                        Some(e) => match e {
                            Element::Paren(Paren::Start) => {
                                depth += 1;
                                expect_num = true;
                            }
                            Element::Number(_) => expect_num = false,
                            _ => return false,
                        },
                    }
                } else {
                    // Expecting an operator or end
                    match iter.next() {
                        None => return depth == 0,
                        Some(e) => match e {
                            Element::Paren(Paren::End) => {
                                depth -= 1;
                                expect_num = false;
                            }
                            Element::Operator(_) => expect_num = true,
                            _ => return false,
                        },
                    }
                }
            }
        }

        /// Evaluates the expression, returning the result if the expression
        /// is valid.
        ///
        /// This uses the operator precedence defined by the `part`.
        fn evaluate(&self, part: &dyn Part) -> AocResult<u64> {
            // First validate
            if !self.is_valid() {
                return Err(AocError::Process(
                    format!("The expression '{}' is malformed", self.original).into(),
                ));
            }

            // Next convert from infix to postfix.
            // This implements the algorithm here:
            // https://www.geeksforgeeks.org/stack-set-2-infix-to-postfix/
            let mut stack = vec![];
            let mut postfix = vec![];
            for e in self.elements.iter() {
                match e {
                    Element::Number(_) => postfix.push(e),
                    Element::Paren(Paren::Start) => stack.push(e),
                    Element::Paren(Paren::End) => loop {
                        match stack.pop() {
                            None => break,
                            Some(se) => {
                                if let Element::Paren(Paren::Start) = se {
                                    break;
                                } else {
                                    postfix.push(se)
                                }
                            }
                        }
                    },
                    Element::Operator(op) => {
                        if let Some(Element::Operator(pop)) = stack.last() {
                            if op.cmp(pop, part).is_le() {
                                postfix.push(stack.pop().unwrap());
                            }
                        }
                        stack.push(e);
                    }
                }
            }
            loop {
                match stack.pop() {
                    None => break,
                    Some(e) => postfix.push(e),
                }
            }
            //println!("Infix: {}", self.original);
            //println!("Postfix: {:?}", postfix);

            // Now evaluate the postfix expressions
            let mut stack = vec![];
            for e in postfix {
                match e {
                    Element::Number(n) => stack.push(*n),
                    Element::Operator(op) => {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(op.evaluate(&a, &b));
                    }
                    _ => panic!(),
                }
            }

            Ok(stack.pop().unwrap())
        }
    }

    /// A list of expressions, which can be parsed from text input.
    pub struct ExpressionList {
        /// The list of expressions.
        expressions: Vec<Expression>,
    }
    impl FromStr for ExpressionList {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                expressions: Expression::gather(s.lines())?,
            })
        }
    }
    impl ExpressionList {
        /// Validates and evaluates every expression and sum the results.
        ///
        /// Each expression uses the operator precedence defined by `part`.
        pub fn evaluation_sum(&self, part: &dyn Part) -> AocResult<u64> {
            process_results(self.expressions.iter().map(|e| e.evaluate(part)), |iter| {
                iter.sum()
            })
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 18,
    name: "Operation Order",
    preprocessor: Some(|input| Ok(Box::new(ExpressionList::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<ExpressionList>()?
                .evaluation_sum(&PartOne)?
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<ExpressionList>()?
                .evaluation_sum(&PartTwo)?
                .into())
        },
    ],
};
