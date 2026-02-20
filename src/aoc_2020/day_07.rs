use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use Answer::Unsigned;
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "light red bags contain 1 bright white bag, 2 muted yellow bags.
dark orange bags contain 3 bright white bags, 4 muted yellow bags.
bright white bags contain 1 shiny gold bag.
muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.
shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.
dark olive bags contain 3 faded blue bags, 4 dotted black bags.
vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.
faded blue bags contain no other bags.
dotted black bags contain no other bags.";
            answers = unsigned![4, 32];
        }
        example {
            input = "shiny gold bags contain 2 dark red bags.
dark red bags contain 2 dark orange bags.
dark orange bags contain 2 dark yellow bags.
dark yellow bags contain 2 dark green bags.
dark green bags contain 2 dark blue bags.
dark blue bags contain 2 dark violet bags.
dark violet bags contain no other bags.";
            answers = &[None, Some(Unsigned(126))];
        }
        actual_answers = unsigned![316, 11310];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use bimap::hash::BiHashMap;
    use derive_new::new;
    use nom::{
        Finish,
        bytes::complete::{is_not, tag, take_until},
        character::complete::space1,
        combinator::map,
        error::context,
        multi::separated_list0,
        sequence::separated_pair,
    };
    use std::collections::HashSet;

    /// Type to use for bag IDs.
    pub type BagId = u32;

    /// Associates a string bag color with an ID for more efficient comparisons.
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
        /// Fetches its ID given a color name, or else creates a new ID for it.
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
        /// Parses the rule from text input.
        fn parse(bag_table: &mut BagTable, input: &str) -> Result<Self, NomParseError> {
            context(
                "bag rule",
                map(
                    separated_pair(
                        take_until(" bags"),
                        tag(" bags contain "),
                        separated_list0(
                            tag(", "),
                            (
                                nom::character::complete::u8,
                                space1,
                                take_until(" bag"),
                                is_not(",."),
                            ),
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
            )
            .parse(input.trim())
            .finish()
            .map(|(_, r)| r)
        }
    }

    /// Set of bag rules, which can be parsed from text input.
    pub struct BagRules {
        /// Table that maps bag IDs to the color names.
        pub bags: BagTable,
        /// Set of rules.
        pub rules: Vec<BagRule>,
    }
    impl FromStr for BagRules {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
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
    }
    impl BagRules {
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

        /// Counts all the bags contained in a particular bag recursively.
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
