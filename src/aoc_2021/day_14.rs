use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
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
    use aoc::parse::{single_alphanumeric, trim};
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
        /// Returns an ordered [`Iterator`] over adjacent pairs of elements in the formula.
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
        /// Returns the pair involved in the insertion.
        fn pair(&self) -> Pair {
            (self.left, self.right)
        }

        /// Returns an [`Iterator`] of the unique elements involved in the insertion.
        fn chars(&self) -> impl Iterator<Item = char> {
            hashset![self.left, self.right, self.insert].into_iter()
        }
    }

    /// The number of occurrences of each element in a formula.
    #[derive(Debug, Clone)]
    pub struct Occurrences {
        /// Map of element characters to the number of times it appears in the formula.
        map: HashMap<char, u64>,
    }
    impl Occurrences {
        /// Creates a new set of occurrences in which every element initially has zero occurrences.
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

        /// Returns the range of numbers of occurrences over all elements.
        pub fn range(&self) -> RangeInclusive<u64> {
            self.map.values().copied().range().unwrap_or(0..=0)
        }
    }
    impl From<char> for Occurrences {
        fn from(c: char) -> Self {
            let mut occurrences = Self::new();
            occurrences.increment(c);
            occurrences
        }
    }
    impl Add for &Occurrences {
        type Output = Occurrences;

        fn add(self, rhs: Self) -> Self::Output {
            let mut map = HashMap::new();
            let chars: HashSet<char> = self.map.keys().chain(rhs.map.keys()).copied().collect();

            for c in chars.into_iter() {
                map.insert(c, self.get(&c) + rhs.get(&c));
            }

            Self::Output { map }
        }
    }
    impl Sum for Occurrences {
        fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.reduce(|o1, o2| &o1 + &o2)
                .unwrap_or_else(Occurrences::new)
        }
    }

    /// Problem definition, which can be parsed from text input.
    #[derive(Debug)]
    pub struct Problem {
        /// The initial polymer template formula.
        template: Formula,
        /// The possible insertions.
        pair_insertions: HashMap<Pair, PairInsertion>,
        /// The set of element characters involved in the whole process.
        chars: HashSet<char>,
    }
    impl FromStr for Problem {
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
    impl Problem {
        /// Returns an [`Iterator`] over all possible pairs of characters.
        fn pairs(&self) -> impl Iterator<Item = Pair> + '_ {
            iproduct!(self.chars.iter().copied(), self.chars.iter().copied())
        }

        /// Returns an [`Iterator`] over the element occurrences at each step as the polymer is built up.
        pub fn builder(&self) -> PolymerBuilder<'_> {
            PolymerBuilder::new(self)
        }
    }

    /// An [`Iterator`] over the occurrences of every element at each step of
    /// the polymer building process.
    pub struct PolymerBuilder<'a> {
        /// The problem for which we are building the polymer.
        problem: &'a Problem,
        /// Map from every possible initial pair of elements to the occurrences of each
        /// element in the expansion of that initial pair at the current step, which
        /// does not include the final element in the expansion.
        occurrence_map: HashMap<Pair, Occurrences>,
    }
    impl<'a> PolymerBuilder<'a> {
        /// Creates a new [`Iterator`] for a given problem.
        fn new(problem: &'a Problem) -> Self {
            // Build the initial occurrence map for every possible pair.
            let occurrence_map = problem.pairs().map(|p| (p, p.0.into())).collect();

            Self {
                problem,
                occurrence_map,
            }
        }
    }
    impl Iterator for PolymerBuilder<'_> {
        type Item = Occurrences;

        fn next(&mut self) -> Option<Self::Item> {
            // Update the map with the next step of expansions.
            // Go through every possible pair of characters
            let mut occurrence_map = self.occurrence_map.clone();
            for (pair, occ) in occurrence_map.iter_mut() {
                if let Some(ins) = self.problem.pair_insertions.get(pair) {
                    *occ = self.occurrence_map.get(&(pair.0, ins.insert)).unwrap()
                        + self.occurrence_map.get(&(ins.insert, pair.1)).unwrap()
                }
            }
            self.occurrence_map = occurrence_map;

            // Now build occurrences for the expanded polymer from the initial template.
            let mut occurrences = self
                .problem
                .template
                .pairs()
                .map(|p| self.occurrence_map.get(&p).unwrap().clone())
                .sum::<Occurrences>();

            // Need to add the last element, which is otherwise not included
            occurrences.increment(*self.problem.template.elements.last().unwrap());
            Some(occurrences)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 14,
    name: "Extended Polymerization",
    preprocessor: Some(|input| Ok(Box::new(Problem::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            let range = input
                .expect_data::<Problem>()?
                .builder()
                .iterations(10)
                .unwrap()
                .range();
            Ok((range.end() - range.start()).into())
        },
        // Part two
        |input| {
            // Process
            let range = input
                .expect_data::<Problem>()?
                .builder()
                .iterations(40)
                .unwrap()
                .range();
            Ok((range.end() - range.start()).into())
        },
    ],
};
