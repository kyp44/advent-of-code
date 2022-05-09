use std::{rc::Rc, str::FromStr};

use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::alphanumeric1,
    combinator::map,
    sequence::{pair, separated_pair},
};

use crate::aoc::{
    parse::{single_alphanumeric, trim},
    prelude::*,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
        vec![Unsigned(123)],
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

#[derive(Clone, new)]
struct Formula {
    elements: Vec<char>,
}
impl Parseable<'_> for Formula {
    fn parser(input: &str) -> NomParseResult<Self> {
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
    fn occurances(&self) -> impl Iterator<Item = Occurance> + '_ {
        self.elements.iter().unique().map(|e| Occurance {
            element: *e,
            number: self.elements.iter().filter_count(|fe| *fe == e),
        })
    }
}

struct Occurance {
    element: char,
    number: usize,
}
impl std::fmt::Debug for Occurance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.element, self.number)
    }
}

#[derive(Debug)]
struct PairInsertion {
    left: char,
    right: char,
    insert: char,
}
impl Parseable<'_> for PairInsertion {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(
            separated_pair(
                pair(single_alphanumeric, single_alphanumeric),
                trim(tag("->")),
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
    fn matches(&self, elements: &(&char, &char)) -> bool {
        self.left == *elements.0 && self.right == *elements.1
    }
}

#[derive(Debug)]
struct PolymerBuilder {
    template: Formula,
    pair_insertions: Box<[PairInsertion]>,
}
impl FromStr for PolymerBuilder {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let secs = s.sections(2)?;

        Ok(Self {
            template: Formula::from_str(secs[0])?,
            pair_insertions: PairInsertion::gather(secs[1].lines())?.into_boxed_slice(),
        })
    }
}
impl PolymerBuilder {
    fn build(&self) -> Polymers {
        Polymers::new(self)
    }
}
struct Polymers<'a> {
    builder: &'a PolymerBuilder,
    formula: Rc<Formula>,
}
impl<'a> Polymers<'a> {
    fn new(builder: &'a PolymerBuilder) -> Self {
        Self {
            builder,
            formula: Rc::new(builder.template.clone()),
        }
    }
}
impl Iterator for Polymers<'_> {
    type Item = Rc<Formula>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut formula = Vec::new();

        // Look for pair matches in a ssliding window
        for elements in self.formula.elements.iter().tuple_windows() {
            match self
                .builder
                .pair_insertions
                .iter()
                .find(|p| p.matches(&elements))
            {
                Some(pair) => {
                    formula.push(pair.left);
                    formula.push(pair.insert);
                }
                None => formula.push(*elements.0),
            }
        }
        // Need to add the last element
        formula.push(*self.formula.elements.last().unwrap());
        self.formula = Rc::new(Formula::new(formula));

        Some(self.formula.clone())
    }
}

pub const SOLUTION: Solution = Solution {
    day: 14,
    name: "Extended Polymerization",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let builder = PolymerBuilder::from_str(input)?;
            let final_formula = builder.build().nth(9).unwrap();

            for formula in builder.build().take(4) {
                println!("{:?}", formula)
            }
            for occurance in final_formula
                .occurances()
                .sorted_unstable_by_key(|o| o.number)
            {
                println!("{:?}", occurance);
            }

            let range = final_formula
                .occurances()
                .map(|o| o.number)
                .range()
                .unwrap();

            // Process
            Ok(Answer::Unsigned(
                (range.end() - range.start()).try_into().unwrap(),
            ))
        },
        // Part b)
        /*|input| {
            // Generation
            let builder = PolymerBuilder::from_str(input)?;
            let final_formula = builder.build().nth(39).unwrap();

            let range = final_formula
                .occurances()
                .map(|o| o.number)
                .range()
                .unwrap();

            // Process
            Ok(Answer::Unsigned(
                (range.end() - range.start()).try_into().unwrap(),
            ))
        },*/
    ],
};
