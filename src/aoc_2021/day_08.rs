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
        let s1 = get_len(2)?;
        let s7 = get_len(3)?;

	fn err(c: char, msg: &str) -> AocError {
	    AocError::Process(format("Problem deducing '{}': {}", c, msg).into());
	}
        fn only_element(c: char, iter: impl Iterator<Item = char>) -> AocResult<char> {
	    let element = iter.next().ok_or_else(|| err(c, "set empty"))?;
	    if iter.next().is_some() {
		return Err(err(c, "set has more than one element"));
	    }
	    Ok(element)
        }
	let map_add = |c: char, mc: char| -> AocResult<char> {
	    //map.insert(c, mc).
	    
	}

        // Deduce which character coorresponds to the variable name characters
        let a = s7.segments.difference(&s1.segments);

        Ok(map)
    }

    fn output_digits(&self) -> AocResult<Box<[u8]>> {
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
