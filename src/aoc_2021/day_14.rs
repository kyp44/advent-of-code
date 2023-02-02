use crate::aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
        vec![Unsigned(2194), Unsigned(2360298895777)],
    "NNCB

CH -> B
HH -> N
CB -> H
NH -> C
HB -> C
HC -> B
HN -> C
NN -> C
BH -> H
NC -> B
NB -> B
BN -> B
BB -> N
BC -> B
CC -> N
CN -> C",
        vec![1588u64, 2188189693529].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use crate::aoc::parse::{single_alphanumeric, trim};
    use itertools::{iproduct, Itertools};
    use maplit::hashset;
    use nom::{
        bytes::complete::tag,
        character::complete::alphanumeric1,
        combinator::map,
        sequence::{pair, separated_pair},
    };
    use std::{
        collections::{HashMap, HashSet},
        iter::Sum,
        ops::{Add, RangeInclusive},
    };

    /// A polymer formula, which can be parsed from text input.
    struct Formula {
        /// The ordered list of elements in the formula.
        elements: Vec<char>,
    }
    impl Parseable<'_> for Formula {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(alphanumeric1, |s: &str| Self {
                elements: s.chars().collect(),
            })(input)
        }
    }
    impl std::fmt::Debug for Formula {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.elements.iter().join(""))
        }
    }
    impl Formula {
        /// Returns an ordered [Iterator] over adjacent pairs of elements in the formula.
        fn pairs(&self) -> impl Iterator<Item = Pair> + '_ {
            self.elements.iter().copied().tuple_windows()
        }
    }

    /// A pair of elements.
    type Pair = (char, char);

    /// An insertion into a polymer formula, which can be parsed from text input.
    #[derive(Debug)]
    struct PairInsertion {
        /// The left element of the pair in which to insert.
        left: char,
        /// The right element of the pair in which to insert.
        right: char,
        /// Element to insert between the pair.
        insert: char,
    }
    impl Parseable<'_> for PairInsertion {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(
                    pair(single_alphanumeric, single_alphanumeric),
                    trim(false, tag("->")),
                    single_alphanumeric,
                ),
                |(lr, insert)| Self {
                    left: lr.0,
                    right: lr.1,
                    insert,
                },
            )(input)
        }
    }
    impl PairInsertion {
        /// The pair involved in the insertion.
        fn pair(&self) -> Pair {
            (self.left, self.right)
        }

        /// Returns an [Iterator] of the unique elements involved in the insertion.
        fn chars(&self) -> impl Iterator<Item = char> {
            hashset![self.left, self.right, self.insert].into_iter()
        }
    }

    /// The occurances of each element in a formula.
    #[derive(Debug, Clone)]
    pub struct Occurances {
        /// Map of element characters to the number of times it appears in the formula.
        map: HashMap<char, u64>,
    }
    impl Occurances {
        /// Creates a new set of occurrences in which every element begins with zero.
        fn new() -> Self {
            Self {
                map: HashMap::new(),
            }
        }

        /// Retrieves the number of occurrences for an element.
        fn get(&self, c: &char) -> u64 {
            match self.map.get(c) {
                Some(o) => *o,
                None => 0,
            }
        }

        /// Increments the number of occurrences for an element.
        fn increment(&mut self, c: char) {
            *self.map.entry(c).or_insert(0) += 1;
        }

        /// The range of numbers of occurrences over all elements.
        pub fn range(&self) -> RangeInclusive<u64> {
            self.map.values().copied().range().unwrap_or(0..=0)
        }
    }
    impl From<char> for Occurances {
        fn from(c: char) -> Self {
            let mut occurances = Self::new();
            occurances.increment(c);
            occurances
        }
    }
    impl Add for &Occurances {
        type Output = Occurances;

        fn add(self, rhs: Self) -> Self::Output {
            let mut map = HashMap::new();
            let chars: HashSet<char> = self.map.keys().chain(rhs.map.keys()).copied().collect();

            for c in chars.into_iter() {
                map.insert(c, self.get(&c) + rhs.get(&c));
            }

            Self::Output { map }
        }
    }
    impl Sum for Occurances {
        fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.reduce(|o1, o2| &o1 + &o2)
                .unwrap_or_else(Occurances::new)
        }
    }

    /// Builder for a polymer, which can be parsed from text input.
    #[derive(Debug)]
    pub struct PolymerBuilder {
        /// The initial polymer template formula.
        template: Formula,
        /// The possible insertions.
        pair_insertions: HashMap<Pair, PairInsertion>,
        /// The set of element characters involved in the whole process.
        chars: HashSet<char>,
    }
    impl FromStr for PolymerBuilder {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let secs = s.sections(2)?;
            let pair_insertions = PairInsertion::gather(secs[1].lines())?;
            let chars = pair_insertions.iter().flat_map(|p| p.chars()).collect();

            Ok(Self {
                template: Formula::from_str(secs[0])?,
                pair_insertions: pair_insertions
                    .into_iter()
                    .map(|ip| (ip.pair(), ip))
                    .collect(),
                chars,
            })
        }
    }
    impl PolymerBuilder {
        pub fn occurrences(&self, nth: usize) -> Occurances {
            // Map of every pair and TODO
            let mut occurances_map: HashMap<(Pair, usize), Occurances> =
                self.pairs().map(|p| ((p, 0), p.0.into())).collect();

            // First, build up occurances for all levels
            for level in 1..=nth {
                // Go through every possible pair of characters
                for pair in self.pairs() {
                    let occurances = match self.pair_insertions.get(&pair) {
                        Some(ip) => {
                            occurances_map
                                .get(&((pair.0, ip.insert), level - 1))
                                .unwrap()
                                + occurances_map
                                    .get(&((ip.insert, pair.1), level - 1))
                                    .unwrap()
                        }
                        None => pair.0.into(),
                    };
                    occurances_map.insert((pair, level), occurances);
                }
            }

            // Now go through the template and add occurances
            let mut occurances = self
                .template
                .pairs()
                .map(|p| occurances_map.get(&(p, nth)).unwrap().clone())
                .sum::<Occurances>();

            // Need to add the last element, which is otherwise not included
            occurances.increment(*self.template.elements.last().unwrap());
            occurances
        }

        /// Returns an [Iterator] over all possible pairs of characters.
        fn pairs(&self) -> impl Iterator<Item = Pair> + '_ {
            iproduct!(self.chars.iter().copied(), self.chars.iter().copied())
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 14,
    name: "Extended Polymerization",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let builder = PolymerBuilder::from_str(input.expect_input()?)?;
            let range = builder.occurrences(10).range();

            // Process
            Ok((range.end() - range.start()).into())
        },
        // Part two
        |input| {
            // Generation
            let builder = PolymerBuilder::from_str(input.expect_input()?)?;
            let range = builder.occurrences(40).range();

            // Process
            Ok((range.end() - range.start()).into())
        },
    ],
};
