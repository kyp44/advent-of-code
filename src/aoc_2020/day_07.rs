use crate::aoc::prelude::*;
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
use std::{collections::HashSet, convert::TryInto};

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

/// Associates a string bag color with an ID for more efficient comparisons
#[derive(Debug, new)]
struct BagTable<'a> {
    #[new(value = "0")]
    next_id: u32,
    #[new(value = "BiHashMap::new()")]
    bimap: BiHashMap<u32, &'a str>,
}

impl<'a> BagTable<'a> {
    fn get_or_add_bag(&mut self, bag_str: &'a str) -> u32 {
        match self.bimap.get_by_right(bag_str) {
            Some(id) => *id,
            None => {
                let id = self.next_id;
                self.bimap.insert(id, bag_str);
                self.next_id += 1;
                id
            }
        }
    }
}

#[derive(Debug)]
struct BagContains {
    count: u32,
    bag_id: u32,
}

#[derive(Debug)]
struct BagRule {
    bag_id: u32,
    contains: Vec<BagContains>,
}

impl BagRule {
    fn parse<'a>(bag_table: &mut BagTable<'a>, input: &'a str) -> Result<Self, NomParseError> {
        context(
            "bag rule",
            map(
                separated_pair(
                    take_until(" bags"),
                    tag(" bags contain "),
                    separated_list0(
                        tag(", "),
                        tuple((
                            nom::character::complete::u32,
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
                            count,
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

struct BagRules<'a> {
    bags: BagTable<'a>,
    rules: Vec<BagRule>,
}

impl<'a> BagRules<'a> {
    fn from_str(s: &'a str) -> Result<Self, NomParseError> {
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

pub const SOLUTION: Solution = Solution {
    day: 7,
    name: "Handy Haversacks",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let mut bag_rules = BagRules::from_str(input.expect_input()?)?;

            // Processing
            let id = bag_rules.bags.get_or_add_bag("shiny gold");
            Ok(Answer::Unsigned({
                let mut containing_bags = HashSet::new();
                containing_bags.insert(id);

                loop {
                    let last_count = containing_bags.len();
                    for rule in bag_rules.rules.iter() {
                        if rule
                            .contains
                            .iter()
                            .any(|cont| containing_bags.contains(&cont.bag_id))
                        {
                            containing_bags.insert(rule.bag_id);
                        }
                    }
                    if containing_bags.len() == last_count {
                        break (last_count - 1).try_into().unwrap();
                    }
                }
            }))
        },
        // Part b)
        |input| {
            // Generation
            let mut bag_rules = BagRules::from_str(input.expect_input()?)?;

            // Processing
            let id = bag_rules.bags.get_or_add_bag("shiny gold");

            fn count_containing_bags(rules: &[BagRule], id: u32) -> u32 {
                match rules.iter().find(|r| r.bag_id == id) {
                    None => 0,
                    Some(rule) => rule
                        .contains
                        .iter()
                        .map(|c| c.count * (1 + count_containing_bags(rules, c.bag_id)))
                        .sum(),
                }
            }
            Ok(Answer::Unsigned(
                count_containing_bags(&bag_rules.rules, id).into(),
            ))
        },
    ],
};
