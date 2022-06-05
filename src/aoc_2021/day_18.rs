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
    vec![Unsigned(123)],
    "[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]",
    vec![11u64].answer_vec()
    }
}

enum Element {
    Open,
    Close,
    Num(u8),
}
struct Number {
    stack: Vec<Element>,
}
impl Parseable<'_> for Number {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        alt((
            map(nom::character::complete::u8, |v| Number {
                stack: vec![Element::Num(v)],
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
                        Element::Num(v) => format!("{}", v),
                    }
                })
                .join(" ")
        )
    }
}
impl Number {
    fn reduce(&mut self) {
        // First find any sufficiently deep nodes with two leaves
        //for node in self.tree.bfs_children_mut().iter {}
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
                    if let Element::Num(v) = e {
                        *v += ln;
                        break;
                    }
                }
                for e in stack[i + 4..].iter_mut() {
                    if let Element::Num(v) = e {
                        *v += rn;
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
}

pub const SOLUTION: Solution = Solution {
    day: 18,
    name: "Snailfish",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let mut numbers = Number::gather(input.lines())?;

            println!("TODO: {:?}", numbers[0]);
            numbers[0].explode();
            println!("TODO: {:?}", numbers[0]);
            numbers[0].explode();
            println!("TODO: {:?}", numbers[0]);
            numbers[0].explode();
            println!("TODO: {:?}", numbers[0]);

            // Process
            Ok(0u64.into())
        },
    ],
};
