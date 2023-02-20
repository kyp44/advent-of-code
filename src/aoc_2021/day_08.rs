use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
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

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::separated;
    use itertools::Itertools;
    use lazy_static::lazy_static;
    use maplit::hashmap;
    use nom::{
        bytes::complete::tag,
        character::complete::{one_of, space1},
        combinator::map,
        multi::{many1, separated_list1},
        sequence::separated_pair,
    };
    use std::collections::{HashMap, HashSet};

    /// The signal patterns for a single digit on a display, which can
    /// be parsed from text input.
    #[derive(PartialEq, Eq)]
    struct Digit {
        /// The set of signal names asserted to create this digit on the display.
        segments: HashSet<char>,
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
        /// Creates a new [`Digit`] based on a mapping from these signal names to
        /// a new set of signal names.
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
        /// Mapping of the of a [`Digit`] using corrected signals to the numeric
        /// digit.
        static ref DIGITS: HashMap<Digit, u8> = hashmap! {
            Digit::from_str("abcefg").unwrap() => 0,
            Digit::from_str("cf").unwrap() => 1,
            Digit::from_str("acdeg").unwrap() => 2,
            Digit::from_str("acdfg").unwrap() => 3,
            Digit::from_str("bcdf").unwrap() => 4,
            Digit::from_str("abdfg").unwrap() => 5,
            Digit::from_str("abdefg").unwrap() => 6,
            Digit::from_str("acf").unwrap() => 7,
            Digit::from_str("abcdefg").unwrap() => 8,
            Digit::from_str("abcdfg").unwrap() => 9,
        };
    }

    /// A set of solved output digits as numbers.
    pub struct OutputDigits {
        /// Ordered digits on the output displays from most significant to
        /// least significant.
        pub digits: Box<[u8]>,
    }
    impl OutputDigits {
        /// Returns the four-digit output as a single number.
        pub fn as_number(&self) -> u64 {
            self.digits
                .iter()
                .zip([1000u64, 100, 10, 1])
                .map(|(d, m)| m * u64::from(*d))
                .sum()
        }
    }

    /// A an entry from your notes about a particular 4-digit display,
    /// which can be parsed from text input.
    pub struct Entry {
        /// The complete set of digits 0 through 9 for this display.
        digits: Box<[Digit]>,
        /// The four output display digits.
        output: Box<[Digit]>,
    }
    impl Parseable<'_> for Entry {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(
                    separated_list1(space1, Digit::parser),
                    separated(tag("|")),
                    separated_list1(space1, Digit::parser),
                ),
                |(digs, out)| Entry {
                    digits: digs.into_boxed_slice(),
                    output: out.into_boxed_slice(),
                },
            )(input)
        }
    }
    impl Entry {
        /// Solve this entry, returning the map of the mixed signal names to the corrected
        /// signal names.
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
                    .ok_or_else(|| {
                        AocError::Process(format!("No sets of length {len} found").into())
                    })
            };
            let d1 = get_len(2)?;
            let d4 = get_len(4)?;
            let d7 = get_len(3)?;
            let d8 = get_len(7)?;

            let mut map_add = |c: char, set: Vec<&char>| -> AocResult<char> {
                /// Sub-function of [`Entry::solve`] that creates an error given a signal name
                /// and message string.
                fn err(c: char, msg: &str) -> AocError {
                    AocError::Process(format!("Problem deducing '{c}': {msg}!").into())
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
            // This procedure was derived ahead of time using interactive Python
            // experimentation.
            // TODO: Should we document the solution process in the notes? Probably should.
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

        /// Solves the entry and returns the output digits as numbers.
        pub fn output_digits(&self) -> AocResult<OutputDigits> {
            let map = self.solve()?;

            Ok(OutputDigits {
                digits: self
                    .output
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
                    .collect::<AocResult<_>>()?,
            })
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 8,
    name: "Seven Segment Search",
    preprocessor: Some(|input| {
        Ok(Box::new(
            input
                .lines()
                .map(|line| Entry::from_str(line)?.output_digits())
                .collect::<AocResult<Vec<_>>>()?,
        )
        .into())
    }),
    solvers: &[
        // Part one
        |input| {
            // Process
            let easy_digits = [1, 4, 7, 8];
            Ok(input
                .expect_data::<Vec<OutputDigits>>()?
                .iter()
                .map(|od| -> u64 { od.digits.iter().filter_count(|d| easy_digits.contains(d)) })
                .sum::<u64>()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Vec<OutputDigits>>()?
                .iter()
                .map(|od| od.as_number())
                .sum::<u64>()
                .into())
        },
    ],
};
