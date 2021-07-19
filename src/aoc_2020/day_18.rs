use std::cmp::Ordering;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{all_consuming, map},
    multi::many1,
};

use crate::aoc::{trim, AocError, AocResult, ParseResult, Parseable, Solution};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![464478013511, 85660197232452],
    "1 + 2 * 3 + 4 * 5 + 6",
    vec![Some(71), Some(231)],
    "1 + (2 * 3) + (4 * (5 + 6))",
    vec![Some(51), Some(51)],
    "2 * 3 + (4 * 5)",
    vec![Some(26), Some(46)],
    "5 + (8 * 3 + 9 + 3 * 4 * 3)",
    vec![Some(437), Some(1445)],
    "5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))",
    vec![Some(12240), Some(669060)],
    "((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2",
    vec![Some(13632), Some(23340)]
    }
}

#[derive(Debug)]
enum Operator {
    Add,
    Mul,
}
impl Operator {
    fn evaluate(&self, a: &u64, b: &u64) -> u64 {
        match self {
            Operator::Add => a + b,
            Operator::Mul => a * b,
        }
    }

    fn cmp<P: Part>(&self, other: &Operator) -> Ordering {
        P::precedence(self).cmp(&P::precedence(other))
    }
}

trait Part {
    fn precedence(op: &Operator) -> u8;
}
struct PartA;
impl Part for PartA {
    fn precedence(op: &Operator) -> u8 {
        match op {
            Operator::Add => 1,
            Operator::Mul => 1,
        }
    }
}
struct PartB;
impl Part for PartB {
    fn precedence(op: &Operator) -> u8 {
        match op {
            Operator::Add => 2,
            Operator::Mul => 1,
        }
    }
}

#[derive(Debug)]
enum Paren {
    Start,
    End,
}

#[derive(Debug)]
enum Element {
    Number(u64),
    Operator(Operator),
    Paren(Paren),
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
                map(trim(digit1), |ds: &str| {
                    Element::Number(ds.parse().unwrap())
                }),
                map(trim(tag("+")), |_| Element::Operator(Operator::Add)),
                map(trim(tag("*")), |_| Element::Operator(Operator::Mul)),
                map(trim(tag("(")), |_| Element::Paren(Paren::Start)),
                map(trim(tag(")")), |_| Element::Paren(Paren::End)),
            ))),
            |elements| Expression {
                original: input,
                elements,
            },
        ))(input)
    }
}
impl Expression<'_> {
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

    fn evaluate<P: Part>(&self) -> AocResult<u64> {
        // First validate
        if !self.is_valid() {
            return Err(AocError::Process(format!(
                "The expression '{}' is malformed",
                self.original
            )));
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
                    if let Some(pe) = stack.last() {
                        if let Element::Operator(pop) = pe {
                            if op.cmp::<P>(pop).is_le() {
                                postfix.push(stack.pop().unwrap());
                            }
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

fn solve<T: Part>(expressions: &Vec<Expression>) -> AocResult<u64> {
    // We have to manually calculate the sum due to the error handling
    let mut s: u64 = 0;
    for e in expressions {
        s += e.evaluate::<T>()?;
    }
    return Ok(s);
}

pub const SOLUTION: Solution = Solution {
    day: 18,
    name: "Operation Order",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let expressions = Expression::gather(input.lines())?;

            // Process
            Ok(solve::<PartA>(&expressions)?)
        },
        // Part b)
        |input| {
            // Generation
            let expressions = Expression::gather(input.lines())?;

            // Process
            Ok(solve::<PartB>(&expressions)?)
        },
    ],
};
