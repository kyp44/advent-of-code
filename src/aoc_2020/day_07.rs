use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(316), Unsigned(11310)],
    "light red bags contain 1 bright white bag, 2 muted yellow bags.
dark orange bags contain 3 bright white bags, 4 muted yellow bags.
bright white bags contain 1 shiny gold bag.
muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.
shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.
dark olive bags contain 3 faded blue bags, 4 dotted black bags.
vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.
faded blue bags contain no other bags.
dotted black bags contain no other bags.",
    vec![4u64, 32].answer_vec(),
    "shiny gold bags contain 2 dark red bags.
dark red bags contain 2 dark orange bags.
dark orange bags contain 2 dark yellow bags.
dark yellow bags contain 2 dark green bags.
dark green bags contain 2 dark blue bags.
dark blue bags contain 2 dark violet bags.
dark violet bags contain no other bags.",
    vec![None, Some(Unsigned(126))]
    }
}

/// Contains solution implementation items.
mod solution {
    use std::collections::HashSet;

    use super::*;
    use bimap::hash::BiHashMap;
    use nom::{
        bytes::complete::{is_not, tag, take_until},
        character::complete::space1,
        combinator::map,
        error::context,
        multi::separated_list0,
        sequence::{separated_pair, tuple},
        Finish,
    };

    /// Type to use for bag IDs.
    pub type BagId = u32;

    /// Associates a string bag color with an ID for more efficient comparisons
    #[derive(Debug, new)]
    pub struct BagTable {
        /// Next ID to issue.
        #[new(value = "0")]
        next_id: BagId,
        /// Map of bag IDs with their color names.
        #[new(value = "BiHashMap::new()")]
        bimap: BiHashMap<BagId, String>,
    }
    impl BagTable {
        /// Given a color name, fetches its ID or else creates a new ID for it.
        pub fn get_or_add_bag(&mut self, bag_str: &str) -> BagId {
            match self.bimap.get_by_right(bag_str) {
                Some(id) => *id,
                None => {
                    let id = self.next_id;
                    self.bimap.insert(id, bag_str.to_string());
                    self.next_id += 1;
                    id
                }
            }
        }
    }

    /// Represents a number of bags contained within a parent bag.
    #[derive(Debug)]
    pub struct BagContains {
        /// Number of bags contained in the parent.
        pub count: usize,
        /// ID of contained bag.
        pub bag_id: BagId,
    }

    /// Rule of a bag, which can be parsed from text input.
    #[derive(Debug)]
    pub struct BagRule {
        /// Bag ID of the parent bag.
        pub bag_id: BagId,
        /// All the bags contained in the parent bag.
        pub contains: Vec<BagContains>,
    }
    impl BagRule {
        /// Parse the rule from text input.
        fn parse(bag_table: &mut BagTable, input: &str) -> Result<Self, NomParseError> {
            context(
                "bag rule",
                map(
                    separated_pair(
                        take_until(" bags"),
                        tag(" bags contain "),
                        separated_list0(
                            tag(", "),
                            tuple((
                                nom::character::complete::u8,
                                space1,
                                take_until(" bag"),
                                is_not(",."),
                            )),
                        ),
                    ),
                    |(bs, vec)| BagRule {
                        bag_id: bag_table.get_or_add_bag(bs),
                        contains: vec
                            .into_iter()
                            .map(|(count, _, bs, _)| BagContains {
                                count: count.into(),
                                bag_id: bag_table.get_or_add_bag(bs),
                            })
                            .collect(),
                    },
                ),
            )(input.trim())
            .finish()
            .map(|(_, r)| r)
        }
    }

    /// Set of bag rules, which can be parsed from text input
    pub struct BagRules {
        /// Table that maps bag IDs to the color names.
        pub bags: BagTable,
        /// Set of rules.
        pub rules: Vec<BagRule>,
    }
    impl BagRules {
        /// Parses the rule set from text input.
        pub fn from_str(s: &str) -> Result<Self, NomParseError> {
            let mut bags = BagTable::new();
            let rules: Vec<BagRule> = s
                .lines()
                .map(|line| BagRule::parse(&mut bags, line))
                .collect::<Result<Vec<BagRule>, NomParseError>>()?;

            // Print things out for testing
            /*
               println!("{}", input);
               println!("{:?}", bag_table);
               for rule in rules.iter() {
               println!("{:?}", rule);
            */

            Ok(BagRules { bags, rules })
        }

        /// Fetches the ID for a bag color name.
        pub fn get_id(&mut self, bag_str: &str) -> BagId {
            self.bags.get_or_add_bag(bag_str)
        }

        /// Counts the number of bags that eventually contain a specific bag.
        pub fn count_containing(&self, id: BagId) -> usize {
            let mut containing_bags = HashSet::new();
            containing_bags.insert(id);

            loop {
                let last_count = containing_bags.len();
                for rule in self.rules.iter() {
                    if rule
                        .contains
                        .iter()
                        .any(|cont| containing_bags.contains(&cont.bag_id))
                    {
                        containing_bags.insert(rule.bag_id);
                    }
                }
                if containing_bags.len() == last_count {
                    break last_count - 1;
                }
            }
        }

        /// Recursively counts all the bags contained in a particular bag.
        pub fn count_contained(&self, id: BagId) -> usize {
            match self.rules.iter().find(|r| r.bag_id == id) {
                None => 0,
                Some(rule) => rule
                    .contains
                    .iter()
                    .map(|c| c.count * (1 + self.count_contained(c.bag_id)))
                    .sum(),
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 7,
    name: "Handy Haversacks",
    preprocessor: Some(|input| {
        let mut bag_rules = BagRules::from_str(input)?;
        let id = bag_rules.get_id("shiny gold");

        Ok(Box::new((bag_rules, id)).into())
    }),
    solvers: &[
        // Part one
        |input| {
            // Processing
            let (bag_rules, id) = input.expect_data::<(BagRules, BagId)>()?;
            Ok(Answer::Unsigned(
                bag_rules.count_containing(*id).try_into().unwrap(),
            ))
        },
        // Part two
        |input| {
            // Processing
            let (bag_rules, id) = input.expect_data::<(BagRules, BagId)>()?;
            Ok(Answer::Unsigned(
                bag_rules.count_contained(*id).try_into().unwrap(),
            ))
        },
    ],
};
