use aoc::prelude::*;
use itertools::Itertools;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]
[[[5,[2,8]],4],[5,[[9,9],0]]]
[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]
[[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]
[[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]
[[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]
[[[[5,4],[7,7]],8],[[8,3],8]]
[[9,3],[[9,9],[6,[4,9]]]]
[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]
[[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]";
            answers = vec![4140u64, 3993].answer_vec();
        }
        actual_answers = vec![Unsigned(4207), Unsigned(4635)];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::map,
        sequence::{delimited, separated_pair},
    };
    use num::Integer;
    use std::{iter::Sum, ops::Add};

    /// An element of a snailfish number.
    #[derive(Clone)]
    enum Element {
        /// An opening bracket.
        Open,
        /// A closing bracket.
        Close,
        /// A number literal.
        Num(u8),
    }

    /// A snailfish number, which can be parsed from text input.
    ///
    /// Note that this was not implemented as nested pairs because the explosion
    /// operation is easier in this form.
    #[derive(Clone)]
    pub struct SnailfishNumber {
        /// Ordered list of the elements of the number.
        stack: Vec<Element>,
    }
    impl Parseable<'_> for SnailfishNumber {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            alt((
                map(nom::character::complete::u8, |n| SnailfishNumber {
                    stack: vec![Element::Num(n)],
                }),
                map(
                    delimited(
                        tag("["),
                        separated_pair(Self::parser, trim(false, tag(",")), Self::parser),
                        tag("]"),
                    ),
                    |(left, right)| {
                        let mut vec = vec![Element::Open];
                        vec.extend(left.stack);
                        vec.extend(right.stack);
                        vec.push(Element::Close);
                        SnailfishNumber { stack: vec }
                    },
                ),
            ))(input)
        }
    }
    impl std::fmt::Debug for SnailfishNumber {
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
                            Element::Num(n) => format!("{n}"),
                        }
                    })
                    .join(" ")
            )
        }
    }
    impl SnailfishNumber {
        /// Reduces the number fully.
        fn reduce(&mut self) {
            loop {
                // Note that, if an explosion is done, the loop will restart due to
                // short circuiting of &&.
                if !self.explode() && !self.split() {
                    break;
                }
            }
        }

        /// Explodes the first applicable number and returns whether an explosion occurred.
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

        /// Splits the first applicable number and returns whether a split occurred.
        fn split(&mut self) -> bool {
            // Look for the first candidate element
            if let Some((i, n)) = self.stack.iter().enumerate().find_map(|(i, e)| {
                if let Element::Num(n) = e
                    && *n >= 10
                {
                    Some((i, *n))
                } else {
                    None
                }
            }) {
                // Now split
                let (a, b) = if n.is_even() {
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

        /// Calculates and returns the magnitude of the number.
        pub fn magnitude(&self) -> u64 {
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
    impl Add for SnailfishNumber {
        type Output = SnailfishNumber;

        fn add(self, rhs: Self) -> Self::Output {
            let mut stack = vec![Element::Open];
            stack.extend(self.stack);
            stack.extend(rhs.stack);
            stack.push(Element::Close);
            let mut number = SnailfishNumber { stack };
            number.reduce();
            number
        }
    }
    impl Sum for SnailfishNumber {
        fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
            match iter.reduce(|a, b| a + b) {
                Some(mut n) => {
                    n.reduce();
                    n
                }
                None => SnailfishNumber { stack: vec![] },
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 18,
    name: "Snailfish",
    preprocessor: Some(|input| Ok(Box::new(SnailfishNumber::gather(input.lines())?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Vec<SnailfishNumber>>()?
                .iter()
                .cloned()
                .sum::<SnailfishNumber>()
                .magnitude()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Vec<SnailfishNumber>>()?
                .iter()
                .permutations(2)
                .map(|pv| (pv[0].clone() + pv[1].clone()).magnitude())
                .max()
                .unwrap()
                .into())
        },
    ],
};
