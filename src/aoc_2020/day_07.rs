use super::super::aoc::{
    ParseError,
    Solution,
};
use nom::{
    Finish,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{digit1, space1},
    combinator::map,
    error::context,
    multi::separated_list0,
    sequence::{separated_pair, tuple},
};
use std::collections::HashSet;
use bimap::hash::BiHashMap;

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn example() {
        let input = "light red bags contain 1 bright white bag, 2 muted yellow bags.
dark orange bags contain 3 bright white bags, 4 muted yellow bags.
bright white bags contain 1 shiny gold bag.
muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.
shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.
dark olive bags contain 3 faded blue bags, 4 dotted black bags.
vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.
faded blue bags contain no other bags.
dotted black bags contain no other bags.";
        
        assert_eq!((SOLUTION.solver)(input).unwrap(), vec![4, 32]);

        let input = "shiny gold bags contain 2 dark red bags.
dark red bags contain 2 dark orange bags.
dark orange bags contain 2 dark yellow bags.
dark yellow bags contain 2 dark green bags.
dark green bags contain 2 dark blue bags.
dark blue bags contain 2 dark violet bags.
dark violet bags contain no other bags.";

        assert_eq!((SOLUTION.solver)(input).unwrap(), vec![0, 126]);
    }

    #[test]
    #[ignore]
    fn actual() {
        assert_eq!(SOLUTION.run(super::super::YEAR_SOLUTIONS.year).unwrap(), vec![316, 11310]);
    }
}

/// Associates a string bag color with an ID for more efficient comparisons
#[derive(Debug)]
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
    fn parse<'a>(bag_table: &mut BagTable<'a>, input: &'a str) -> Result<Self, ParseError> {
        context(
            "bag rule",
            map(
                separated_pair(
                    take_until(" bags"),
                    tag(" bags contain "),
                    separated_list0(
                        tag(", "),
                        tuple((
                            digit1,
                            space1,
                            take_until(" bag"),
                            is_not(",."),
                        ))
                    ),
                ),
                |(bs, vec)| {
                    BagRule {
                        bag_id: bag_table.get_or_add_bag(bs),
                        contains: vec.iter().map(|(ids, _, bs, _)| {
                            BagContains {
                                count: ids.parse().unwrap(),
                                bag_id: bag_table.get_or_add_bag(bs),
                            }
                        }).collect(),
                    }
                }
            )
        )(input.trim()).finish().map(|(_, r)| r)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 7,
    name: "Handy Haversacks",
    solver: |input| {
        // Generation
        let mut bag_table = BagTable::new();
        let rules: Vec<BagRule> = input.trim_end().lines()
            .map(|line| BagRule::parse(&mut bag_table, line))
            .collect::<Result<Vec<BagRule>, ParseError>>()?;

        // Print things out for testing
        /*
        println!("{}", input);
        println!("{:?}", bag_table);
        for rule in rules.iter() {
            println!("{:?}", rule);
        } */

        // Processing
        let id = bag_table.get_or_add_bag("shiny gold");

        // Part a)
        let num_containers = {
            let mut containing_bags = HashSet::new();
            containing_bags.insert(id);

            loop {
                let last_count = containing_bags.len();
                for rule in rules.iter() {
                    if rule.contains.iter().any(|cont| containing_bags.contains(&cont.bag_id)) {
                        containing_bags.insert(rule.bag_id);
                    }
                }
                if containing_bags.len() == last_count {
                    break (last_count - 1) as u32;
                }
            }
        };

        // Part b)
        fn count_containing_bags(rules: &[BagRule], id: u32) -> u32 {
            match rules.iter().find(|r| r.bag_id == id) {
                None => 0,
                Some(rule) => {
                    rule.contains.iter().map(|c| c.count*(1 + count_containing_bags(rules, c.bag_id))).sum()
                }
            }
        }
        let containing_count = count_containing_bags(&rules, id);
        
        let answers = vec![
            num_containers.into(),
            containing_count.into(),
        ];
        
        Ok(answers)
    }
};
