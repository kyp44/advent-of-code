use std::collections::{HashMap, HashSet};

use itertools::{process_results, Itertools};
use nom::{
    bytes::complete::tag,
    character::complete::{one_of, space1},
    combinator::map,
    multi::{many1, separated_list1},
    sequence::separated_pair,
};

use crate::aoc::{parse::separated, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(123)],
    "be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb | fdgacbe cefdb cefbgd gcbe
edbfga begcd cbg gc gcadebf fbgde acbgfd abcde gfcbed gfec | fcgedb cgb dgebacf gc
fgaebd cg bdaec gdafb agbcfd gdcbef bgcad gfac gcb cdgabef | cg cg fdcagb cbg
fbegcd cbd adcefb dageb afcb bc aefdc ecdab fgdeca fcdbega | efabcd cedba gadfec cb
aecbfdg fbg gf bafeg dbefa fcge gcbea fcaegb dgceab fcbdga | gecf egdcabf bgf bfgea
fgeab ca afcebg bdacfeg cfaedg gcfdb baec bfadeg bafgc acf | gebdcfa ecba ca fadegcb
dbcfg fgd bdegcaf fgec aegbdf ecdfab fbedc dacgb gdcebf gf | cefg dcbef fcge gbcadfe
bdfegc cbegaf gecbf dfcage bdacg ed bedf ced adcbefg gebcd | ed bcgafe cdgba cbgef
egadfb cdbfeg cegd fecab cgb gbdefca cg fgcdab egfdb bfceg | gbdfcae bgc cg cgb
gcafb gcf dcaebfg ecagb gf abcdeg gaef cafbge fdbac fegbdc | fgae cfgab fg bagce",
    vec![26u64].answer_vec()
    }
}

#[derive(PartialEq, Eq)]
struct Digit {
    segments: HashSet<char>,
}
impl Parseable<'_> for Digit {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(many1(one_of("abcdefg")), |chars| Digit {
            segments: chars.into_iter().collect(),
        })(input)
    }
}
impl std::hash::Hash for Digit {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for e in self.segments.iter().sorted() {
            e.hash(state);
        }
    }
}
impl Digit {
    fn map(&self, map: HashMap<char, char>) -> Self {
        Digit {
            segments: self
                .segments
                .iter()
                .map(|c| match map.get(c) {
                    Some(mc) => *mc,
                    None => *c,
                })
                .collect(),
        }
    }
}

lazy_static! {
    static ref DIGITS: HashMap<Digit, u8> = {
        let mut digits = HashMap::new();
        digits.insert(Digit::from_str("abcefg").unwrap(), 0);
        digits.insert(Digit::from_str("cf").unwrap(), 1);
        digits.insert(Digit::from_str("acdeg").unwrap(), 2);
        digits.insert(Digit::from_str("acdfg").unwrap(), 3);
        digits.insert(Digit::from_str("bcdf").unwrap(), 4);
        digits.insert(Digit::from_str("abdfg").unwrap(), 5);
        digits.insert(Digit::from_str("abdefg").unwrap(), 6);
        digits.insert(Digit::from_str("acf").unwrap(), 7);
        digits.insert(Digit::from_str("abcdefg").unwrap(), 8);
        digits.insert(Digit::from_str("abcdfg").unwrap(), 9);
        digits
    };
}

struct Line {
    digits: Box<[Digit]>,
    output: Box<[Digit]>,
}
impl Parseable<'_> for Line {
    fn parser(input: &str) -> NomParseResult<Self> {
        let chars = "abcdefg";

        map(
            separated_pair(
                separated_list1(space1, Digit::parser),
                separated(tag("|")),
                separated_list1(space1, Digit::parser),
            ),
            |(digs, out)| Line {
                digits: digs.into_boxed_slice(),
                output: out.into_boxed_slice(),
            },
        )(input)
    }
}
impl Line {
    fn solve(&self) -> AocResult<HashMap<char, char>> {
        // First verify the number of digits
        if self.digits.len() != 10 {
            return Err(AocError::Process(
                format!(
                    "A line has {} digit segments instead of 10",
                    self.digits.len()
                )
                .into(),
            ));
        }
        let mut map = HashMap::new();

        // Some known digits based on their lengths
        let get_len = |len: usize| {
            self.digits
                .iter()
                .filter_map(|d| {
                    if d.segments.len() == len {
                        Some(d)
                    } else {
                        None
                    }
                })
                .next()
                .ok_or_else(|| AocError::Process(format!("No sets of length {} found", len).into()))
        };
        let d1 = get_len(2)?;
        let d4 = get_len(4)?;
        let d7 = get_len(3)?;
        let d8 = get_len(7)?;

        fn err(c: char, msg: &str) -> AocError {
            AocError::Process(format!("Problem deducing '{}': {}!", c, msg).into())
        }
        fn only_element<'a>(c: char, mut iter: impl Iterator<Item = &'a char>) -> AocResult<char> {
            let element = iter.next().ok_or_else(|| err(c, "set empty"))?;
            if iter.next().is_some() {
                return Err(err(c, "set has more than one element"));
            }
            Ok(*element)
        }
        let mut map_add = |c: char, mc: char| -> AocResult<char> {
            match map.insert(c, mc) {
                Some(_) => Err(err(c, "map already exists")),
                None => Ok(mc),
            }
        };
        let length_intersection = |len: usize| -> HashSet<char> {
            self.digits
                .iter()
                .filter_map(|d| {
                    if d.segments.len() == len {
                        Some(d.segments.clone())
                    } else {
                        None
                    }
                })
                .reduce(|a, b| a.intersection(&b).map(|c| *c).collect())
                .unwrap_or_else(|| HashSet::new())
        };

        // Deduce which character corresponds to the variable name characters
        // This procedure was derived ahead of time usingn Python experimentation
        let a = map_add(
            'a',
            only_element('a', d7.segments.difference(&d1.segments))?,
        )?;
        let is5 = length_intersection(5);
        let is6 = length_intersection(6);
        let g = map_add(
            'g',
            only_element('g', is5.intersection(&is6).filter(|c| **c != a))?,
        )?;
        let d = map_add(
            'd',
            only_element('d', is5.difference(&HashSet::from([a, g])))?,
        )?;
        let f = map_add('f', only_element('f', is6.intersection(&d1.segments))?)?;
        let c = map_add(
            'c',
            only_element('c', d1.segments.iter().filter(|c| **c != f))?,
        )?;
        let b = map_add(
            'b',
            only_element('b', d4.segments.difference(&HashSet::from([c, d, f])))?,
        )?;
        map_add(
            'e',
            only_element(
                'e',
                d8.segments.difference(&HashSet::from([a, b, c, d, f, g])),
            )?,
        )?;

        Ok(map)
    }

    fn output_digits(&self) -> AocResult<Box<[u8]>> {
        let map = self.solve()?;

        todo!()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 8,
    name: "Seven Segment Search",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let lines = Line::gather(input.lines())?;

            println!("Map: {:?}", lines[0].solve()?);
            // Process
            Ok(0u64.into())
        },
    ],
};
