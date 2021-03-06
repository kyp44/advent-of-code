use std::collections::{HashMap, HashSet};

use itertools::Itertools;
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
    vec![Unsigned(330), Unsigned(1010472)],
    "acedgfb cdfbe gcdfa fbcad dab cefabd cdfgeb eafb cagedb ab | cdfeb fcadb cdfeb cdbaf",
    vec![0u64, 5353].answer_vec(),
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
    vec![26u64, 61229].answer_vec()
    }
}

#[derive(Eq)]
struct Digit {
    segments: HashSet<char>,
}

impl PartialEq for Digit {
    fn eq(&self, other: &Self) -> bool {
        self.segments == other.segments
    }
}
impl Parseable<'_> for Digit {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
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
    fn map(&self, map: &HashMap<char, char>) -> Self {
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
    fn parser(input: &str) -> NomParseResult<&str, Self> {
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
                .find(|d| d.segments.len() == len)
                .ok_or_else(|| AocError::Process(format!("No sets of length {} found", len).into()))
        };
        let d1 = get_len(2)?;
        let d4 = get_len(4)?;
        let d7 = get_len(3)?;
        let d8 = get_len(7)?;

        let mut map_add = |c: char, set: Vec<&char>| -> AocResult<char> {
            fn err(c: char, msg: &str) -> AocError {
                AocError::Process(format!("Problem deducing '{}': {}!", c, msg).into())
            }
            if set.len() != 1 {
                return Err(err(c, "set does not have exactly one element"));
            }
            let mc = *set[0];
            match map.insert(mc, c) {
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
                .reduce(|a, b| a.intersection(&b).copied().collect())
                .unwrap_or_default()
        };

        // Deduce which character corresponds to the variable name characters
        // This procedure was derived ahead of time usingn Python experimentation
        let a = map_add('a', d7.segments.difference(&d1.segments).collect())?;
        let is5 = length_intersection(5);
        let is6 = length_intersection(6);
        let g = map_add('g', is5.intersection(&is6).filter(|c| **c != a).collect())?;
        let d = map_add('d', is5.difference(&HashSet::from([a, g])).collect())?;
        let f = map_add('f', is6.intersection(&d1.segments).collect())?;
        let c = map_add('c', d1.segments.iter().filter(|c| **c != f).collect())?;
        let b = map_add(
            'b',
            d4.segments.difference(&HashSet::from([c, d, f])).collect(),
        )?;
        map_add(
            'e',
            d8.segments
                .difference(&HashSet::from([a, b, c, d, f, g]))
                .collect(),
        )?;

        Ok(map)
    }

    fn output_digits(&self) -> AocResult<Box<[u8]>> {
        let map = self.solve()?;

        self.output
            .iter()
            .map(|d| {
                let mapped = d.map(&map);
                DIGITS
                    .get(&mapped)
                    .ok_or_else(|| {
                        AocError::Process(
                            format!(
                                "Mapped segments '{}', not a valid digit!",
                                mapped.segments.iter().collect::<String>()
                            )
                            .into(),
                        )
                    })
                    .map(|v| *v)
            })
            .collect()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 8,
    name: "Seven Segment Search",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let lines = Line::gather(input.expect_input()?.lines())?;
            let count_digits = vec![1, 4, 7, 8];
            let num_digits = lines
                .into_iter()
                .map(|line| {
                    Ok(line
                        .output_digits()?
                        .iter()
                        .filter_count(|d| count_digits.contains(d)))
                })
                .collect::<AocResult<Vec<usize>>>()?;

            // Process
            Ok(Answer::Unsigned(
                num_digits.into_iter().sum::<usize>().try_into().unwrap(),
            ))
        },
        // Part b)
        |input| {
            // Generation
            let lines = Line::gather(input.expect_input()?.lines())?;
            let num_digits = lines
                .into_iter()
                .map(|line| {
                    Ok(line
                        .output_digits()?
                        .iter()
                        .zip([1000u64, 100, 10, 1])
                        .map(|(d, m)| m * u64::from(*d))
                        .sum())
                })
                .collect::<AocResult<Vec<u64>>>()?;

            // Process
            Ok(num_digits.into_iter().sum::<u64>().into())
        },
    ],
};
