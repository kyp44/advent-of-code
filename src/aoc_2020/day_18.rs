use crate::aoc::prelude::*;
use crate::aoc::{parse::trim, SolverReturn};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map},
    multi::many1,
};
use std::cmp::Ordering;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(464478013511), Unsigned(85660197232452)],
    "1 + 2 * 3 + 4 * 5 + 6",
    vec![71u64, 231].answer_vec(),
    "1 + (2 * 3) + (4 * (5 + 6))",
    vec![51u64, 51].answer_vec(),
    "2 * 3 + (4 * 5)",
    vec![26u64, 46].answer_vec(),
    "5 + (8 * 3 + 9 + 3 * 4 * 3)",
    vec![437u64, 1445].answer_vec(),
    "5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))",
    vec![12240u64, 669060].answer_vec(),
    "((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2",
    vec![13632u64, 23340].answer_vec()
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

    fn cmp(&self, other: &Operator, part: &dyn Part) -> Ordering {
        part.precedence(self).cmp(&part.precedence(other))
    }
}

trait Part {
    fn precedence(&self, op: &Operator) -> u8;
}
struct PartA;
impl Part for PartA {
    fn precedence(&self, op: &Operator) -> u8 {
        match op {
            Operator::Add => 1,
            Operator::Mul => 1,
        }
    }
}
struct PartB;
impl Part for PartB {
    fn precedence(&self, op: &Operator) -> u8 {
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
    fn parser(input: &'a str) -> NomParseResult<&str, Self> {
        all_consuming(map(
            many1(alt((
                map(trim(nom::character::complete::u64), Element::Number),
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

fn solve<'a>(expressions: &[Expression], part: &dyn Part) -> AocResult<SolverReturn<'a>> {
    // We have to manually calculate the sum due to the error handling
    let mut s: u64 = 0;
    for e in expressions {
        s += e.evaluate(part)?;
    }
    Ok(s.into())
}

pub const SOLUTION: Solution = Solution {
    day: 18,
    name: "Operation Order",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let expressions = Expression::gather(input.expect_input()?.lines())?;

            // Process
            solve(&expressions, &PartA)
        },
        // Part b)
        |input| {
            // Generation
            let expressions = Expression::gather(input.expect_input()?.lines())?;

            // Process
            solve(&expressions, &PartB)
        },
    ],
};
