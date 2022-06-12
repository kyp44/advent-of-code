use std::{iter::Sum, ops::Add};

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    sequence::{delimited, separated_pair},
};

use crate::aoc::{parse::trim, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(4207), Unsigned(4635)],
    "[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]
[[[5,[2,8]],4],[5,[[9,9],0]]]
[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]
[[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]
[[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]
[[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]
[[[[5,4],[7,7]],8],[[8,3],8]]
[[9,3],[[9,9],[6,[4,9]]]]
[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]
[[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]",
    vec![4140u64, 3993].answer_vec()
    }
}

#[derive(Clone)]
enum Element {
    Open,
    Close,
    Num(u8),
}
#[derive(Clone)]
struct Number {
    stack: Vec<Element>,
}
impl Parseable<'_> for Number {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        alt((
            map(nom::character::complete::u8, |n| Number {
                stack: vec![Element::Num(n)],
            }),
            map(
                delimited(
                    tag("["),
                    separated_pair(Self::parser, trim(tag(",")), Self::parser),
                    tag("]"),
                ),
                |(left, right)| {
                    let mut vec = vec![Element::Open];
                    vec.extend(left.stack);
                    vec.extend(right.stack);
                    vec.push(Element::Close);
                    Number { stack: vec }
                },
            ),
        ))(input)
    }
}
impl std::fmt::Debug for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            self.stack
                .iter()
                .map(|e| {
                    match e {
                        Element::Open => "[".to_string(),
                        Element::Close => "]".to_string(),
                        Element::Num(n) => format!("{}", n),
                    }
                })
                .join(" ")
        )
    }
}
impl Number {
    fn reduce(&mut self) {
        loop {
            if !self.explode() && !self.split() {
                break;
            }
        }
    }

    /// Look for a pair to explode and returns whether this was done or not.
    fn explode(&mut self) -> bool {
        let pairs: Vec<(usize, u8, u8)> = self
            .stack
            .iter()
            .tuple_windows()
            .enumerate()
            .filter_map(|(i, (a, b, c, d))| {
                if matches!(a, Element::Open) && matches!(d, Element::Close) {
                    if let (Element::Num(ln), Element::Num(rn)) = (b, c) {
                        Some((i, *ln, *rn))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        for (i, ln, rn) in pairs {
            // Found a purely numeric pair
            if self.stack.iter().take(i).fold(0, |a, e| match e {
                Element::Open => a + 1,
                Element::Close => a - 1,
                Element::Num(_) => a,
            }) >= 4
            {
                // The pair is sufficiently deep so explode it!
                // Add the pair numbers to the adjacent numbers if they exist
                let stack = self.stack.as_mut_slice();
                for e in stack[0..i].iter_mut().rev() {
                    if let Element::Num(n) = e {
                        *n += ln;
                        break;
                    }
                }
                for e in stack[i + 4..].iter_mut() {
                    if let Element::Num(n) = e {
                        *n += rn;
                        break;
                    }
                }
                // Now replace the pair
                self.stack.splice(i..i + 4, [Element::Num(0)]);

                return true;
            }
        }
        false
    }

    /// Split first applicable number and returns whether a split occurred.
    fn split(&mut self) -> bool {
        // Look for the first candidate element
        if let Some((i, n)) = self.stack.iter().enumerate().find_map(|(i, e)| {
            if let Element::Num(n) = e && *n >= 10 {
                Some((i, *n))
            } else {
                None
            }
        }) {
            // Now split
            let (a, b) = if n % 2 == 0 {
                (n / 2, n / 2)
            } else {
                (n / 2, n / 2 + 1)
            };
            self.stack.splice(
                i..i + 1,
                [
                    Element::Open,
                    Element::Num(a),
                    Element::Num(b),
                    Element::Close,
                ],
            );
            true
        } else {
            false
        }
    }

    fn magnitude(&self) -> u64 {
        // Convert to postfix
        let mut stack = vec![];
        for e in self.stack.iter() {
            match e {
                Element::Open => {}
                Element::Close => {
                    let v = 2 * stack.pop().unwrap() + 3 * stack.pop().unwrap();
                    stack.push(v);
                }
                Element::Num(n) => stack.push((*n).into()),
            }
        }
        stack[0]
    }
}
impl Add for Number {
    type Output = Number;

    fn add(self, rhs: Self) -> Self::Output {
        let mut stack = vec![Element::Open];
        stack.extend(self.stack);
        stack.extend(rhs.stack);
        stack.push(Element::Close);
        let mut number = Number { stack };
        number.reduce();
        number
    }
}
impl Sum for Number {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        match iter.reduce(|a, b| a + b) {
            Some(mut n) => {
                n.reduce();
                n
            }
            None => Number { stack: vec![] },
        }
    }
}

pub const SOLUTION: Solution = Solution {
    day: 18,
    name: "Snailfish",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let numbers = Number::gather(input.expect_input()?.lines())?;

            // Process
            Ok(numbers.into_iter().sum::<Number>().magnitude().into())
        },
        // Part b)
        |input| {
            // Generation
            let numbers = Number::gather(input.expect_input()?.lines())?;

            // Process
            Ok(numbers
                .iter()
                .permutations(2)
                .map(|pv| (pv[0].clone() + pv[1].clone()).magnitude())
                .max()
                .unwrap()
                .into())
        },
    ],
};
