use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    str::FromStr,
};

use itertools::Itertools;
use maplit::hashset;
use nom::{
    bytes::complete::tag, character::complete::alphanumeric1, combinator::map,
    sequence::separated_pair,
};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
        vec![Unsigned(123)],
    "start-A
start-b
A-c
A-b
b-d
A-end
b-end",
        vec![123u64].answer_vec()
        }
}

#[derive(PartialEq, Eq, Hash)]
enum CaveType {
    START,
    END,
    BIG,
    SMALL,
}
impl From<&str> for CaveType {
    fn from(s: &str) -> Self {
        match s {
            "start" => Self::START,
            "end" => Self::END,
            s if s.to_lowercase() == s => Self::SMALL,
            _ => Self::BIG,
        }
    }
}

#[derive(Eq)]
struct Cave<'a> {
    name: &'a str,
    node_type: CaveType,
    connected: Vec<Rc<Self>>,
}
impl PartialEq for Cave<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl std::hash::Hash for Cave<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
impl<'a> Cave<'a> {
    fn new(name: &'a str) -> Self {
        Self {
            name,
            node_type: name.into(),
            connected: Vec::new(),
        }
    }

    fn add_connection(&mut self, cave: Rc<Self>) {
        self.connected.push(cave);
    }

    fn has_name(&self, name: &str) -> bool {
        self.name == name
    }
}

struct CaveSystem<'a> {
    start: Rc<Cave<'a>>,
}
impl FromStr for CaveSystem<'_> {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        struct Passage<'b> {
            caves: [&'b str; 2],
        }
        impl<'b> Parseable<'b> for Passage<'b> {
            fn parser(input: &'b str) -> NomParseResult<Self> {
                map(
                    separated_pair(alphanumeric1, tag("-"), alphanumeric1),
                    |(a, b)| Self { caves: [a, b] },
                )(input)
            }
        }
        impl Passage<'_> {
            fn connected(&self, from: &str) -> Option<&str> {
                let [a, b] = self.caves;
                if from == a {
                    Some(b)
                } else if from == b {
                    Some(a)
                } else {
                    None
                }
            }
        }

        let passages = Passage::gather(s.lines())?;

        // First create all the Caves
        let mut cave_map: HashMap<&str, Rc<Cave>> = passages
            .iter()
            .flat_map(|p| p.caves.iter())
            .unique()
            .map(|name| (*name, Rc::new(Cave::new(name))))
            .collect();

        // Now buid the connections for each cave
        // TODO: This doesn't compile due to mutability, need a better solution
        /*for (name, cave) in cave_map.iter_mut() {
            for conn_name in passages.iter().filter_map(|p| p.connected(name)).unique() {
                cave.add_connection(cave_map.get_mut(conn_name).unwrap().clone());
            }
        }*/

        todo!()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 12,
    name: "Passage Pathing",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let cave_system = CaveSystem::from_str(input)?;

            // Process
            Ok(0u64.into())
        },
    ],
};
