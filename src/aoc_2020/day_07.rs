use super::super::aoc::{
    ParseResult,
    Solution,
};
use nom::{
    Finish,
    branch::alt,
    bytes::complete::is_not,
    character::complete::{line_ending, space0, space1},
    combinator::{all_consuming, map},
    error::context,
    multi::separated_list1,
    sequence::{pair, tuple},
};
use bimap::hash::BiHashMap;

#[cfg(test)]
mod tests{
    use super::*;
    use crate::solution_test;

    solution_test! {
        "light red bags contain 1 bright white bag, 2 muted yellow bags.
dark orange bags contain 3 bright white bags, 4 muted yellow bags.
bright white bags contain 1 shiny gold bag.
muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.
shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.
dark olive bags contain 3 faded blue bags, 4 dotted black bags.
vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.
faded blue bags contain no other bags.
dotted black bags contain no other bags.",
        vec![11],
        vec![]
    }
}

/// Associates a string bag color with an ID for more efficient comparisons
struct BagTable<'a> {
    next_id: u32,
    bimap: BiHashMap<u32, &'a str>,
}

impl<'a> BagTable<'a> {
    fn new() -> Self {
        BagTable { next_id: 0, bimap: BiHashMap::new() }
    }
    
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

    fn get_bag(&self, bag_id: u32) -> Option<&str> {
        self.bimap.get_by_left(&bag_id).map(|s| *s)
    }
}

struct BagfContains {
    count: u32,
    bag_id: u32,
}
struct BagRule<'a> {
    bag_table: &'a BagTable,
    bag_id: u32,
    contains: Vec<BagContains>,
}

impl BagRule {
    fn parse(bag_table: &BagTable, input: &str) -> Result<Self, ParseError> {
        context(
            "bag rule",
            separated_pair(
                
            )
        )(input.trim()).map(|(_, r)| r)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 7,
    name: "Handy Haversacks",
    solver: |input| {
        // Generation

        // Processing
        let answers = vec![];
        
        Ok(answers)
    }
};
