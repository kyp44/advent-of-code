use std::fmt::Display;
use std::ops::Add;
use std::ops::Mul;

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{all_consuming, map},
    multi::many1,
};

use crate::aoc::{AocError, ParseResult, Parseable, Solution};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![464478013511],
    "1 + 2 * 3 + 4 * 5 + 6",
    vec![Some(71)],
    "1 + (2 * 3) + (4 * (5 + 6))",
    vec![Some(51)],
    "2 * 3 + (4 * 5)",
    vec![Some(26)],
    "5 + (8 * 3 + 9 + 3 * 4 * 3)",
    vec![Some(437)],
    "5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))",
    vec![Some(12240)],
    "((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2",
    vec![Some(13632)]
    }
}

enum ElementType {
    Number,
    Operator,
    Parenthesis,
}
impl Display for ElementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ElementType::*;
        f.write_str(match self {
            Number => "number",
            Operator => "operator",
            Parenthesis => "parenthesis",
        })
    }
}

#[derive(Debug)]
enum Element {
    Number(u64),
    Add,
    Mult,
    ParenStart,
    ParenEnd,
}
impl Element {
    fn unexpected(&self, expr: &str, expected: &[ElementType]) -> AocError {
        AocError::Process(format!(
            "Expected a {} but found a {} instead in expression '{}'",
            expected.iter().map(|et| format!("{}", et)).join(" or "),
            self.element_type(),
            expr
        ))
    }

    fn element_type(&self) -> ElementType {
        use Element::*;
        match self {
            Number(_) => ElementType::Number,
            Add | Mult => ElementType::Operator,
            ParenStart | ParenEnd => ElementType::Parenthesis,
        }
    }
}

#[derive(Debug)]
struct Expression<'a> {
    original: &'a str,
    elements: Vec<Element>,
}
impl<'a> Parseable<'a> for Expression<'a> {
    fn parser(input: &'a str) -> ParseResult<Self> {
        all_consuming(map(
            many1(alt((
                map(digit1, |ds: &str| Element::Number(ds.parse().unwrap())),
                map(tag(" + "), |_| Element::Add),
                map(tag(" * "), |_| Element::Mult),
                map(tag("("), |_| Element::ParenStart),
                map(tag(")"), |_| Element::ParenEnd),
            ))),
            |elements| Expression {
                original: input,
                elements,
            },
        ))(input)
    }
}
impl Expression<'_> {
    fn evaluate(&self) -> Result<u64, AocError> {
        fn eval<'a>(
            expr: &str,
            iter: &mut impl Iterator<Item = &'a Element>,
            parens: bool,
        ) -> Result<u64, AocError> {
            type OperatorFunc = fn(u64, u64) -> u64;
            enum EvalItem {
                Number(u64),
                Operator(OperatorFunc),
            }
            impl EvalItem {
                fn expect_number(&self) -> u64 {
                    match self {
                        EvalItem::Number(n) => *n,
                        _ => panic!(),
                    }
                }

                fn expect_operator(&self) -> OperatorFunc {
                    match self {
                        EvalItem::Operator(f) => *f,
                        _ => panic!(),
                    }
                }
            }

            let abrupt = || AocError::Process(format!("Expression '{}' ended abruptly", expr));
            let mut eval_stack = vec![];
            loop {
                // Expecting a number (or sub-expression)
                let next = iter.next().ok_or_else(abrupt)?;
                match next {
                    Element::Number(n) => eval_stack.push(EvalItem::Number(*n)),
                    Element::ParenStart => {
                        eval_stack.push(EvalItem::Number(eval(expr, iter, true)?))
                    }
                    _ => {
                        return Err(
                            next.unexpected(expr, &[ElementType::Number, ElementType::Parenthesis])
                        )
                    }
                }

                // Is there something to evaluate?
                if eval_stack.len() > 1 {
                    let v2 = eval_stack.pop().unwrap().expect_number();
                    let f = eval_stack.pop().unwrap().expect_operator();
                    let v1 = eval_stack.pop().unwrap().expect_number();
                    eval_stack.push(EvalItem::Number(f(v1, v2)));
                }

                // Now expecting an operator or the end
                match iter.next() {
                    None => {
                        if parens {
                            return Err(abrupt());
                        } else {
                            return Ok(eval_stack[0].expect_number());
                        }
                    }
                    Some(e) => match e {
                        Element::Add => eval_stack.push(EvalItem::Operator(u64::add)),
                        Element::Mult => eval_stack.push(EvalItem::Operator(u64::mul)),
                        Element::ParenEnd => {
                            if parens {
                                return Ok(eval_stack[0].expect_number());
                            } else {
                                return Err(AocError::Process(format!(
                                    "Did not expect an end {} in expression '{}'",
                                    ElementType::Parenthesis,
                                    expr,
                                )));
                            }
                        }
                        _ => {
                            return Err(e.unexpected(
                                expr,
                                &[ElementType::Operator, ElementType::Parenthesis],
                            ))
                        }
                    },
                }
            }
        }
        eval(self.original, &mut self.elements.iter(), false)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 18,
    name: "Operation Order",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let expressions = Expression::gather(input.lines())?;

            // We have to manually calculate the sum due to the error handling
            let mut s: u64 = 0;
            for e in expressions {
                s += e.evaluate()?;
            }
            Ok(s)
        },
    ],
};
