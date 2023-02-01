use crate::aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
        vec![Unsigned(4011), Unsigned(108035)],
    "start-A
start-b
A-c
A-b
b-d
A-end
b-end",
        vec![10u64, 36].answer_vec(),
    "dc-end
HN-start
start-kj
dc-start
dc-HN
LN-dc
HN-end
kj-sa
kj-HN
kj-dc",
    vec![19u64, 103].answer_vec(),
    "fs-end
he-DX
fs-he
start-DX
pj-DX
end-zg
zg-sl
zg-pj
pj-he
RW-he
fs-DX
pj-RW
zg-RW
start-pj
he-WI
zg-he
pj-fs
start-RW",
    vec![226u64, 3509].answer_vec()
        }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use infinitable::Infinitable;
    use nom::{
        bytes::complete::tag, character::complete::alphanumeric1, combinator::map,
        sequence::separated_pair,
    };
    use petgraph::prelude::*;
    use std::{
        collections::{HashMap, HashSet},
        fmt::Debug,
    };

    #[derive(PartialEq, Eq, Hash, Copy, Clone)]
    enum CaveType {
        Start,
        End,
        Big,
        Small,
    }
    impl From<&str> for CaveType {
        fn from(s: &str) -> Self {
            match s {
                "start" => Self::Start,
                "end" => Self::End,
                s if s.to_lowercase() == s => Self::Small,
                _ => Self::Big,
            }
        }
    }

    #[derive(Eq)]
    pub struct Cave {
        name: String,
        cave_type: CaveType,
    }
    impl Debug for Cave {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.name)
        }
    }
    impl Cave {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                cave_type: name.into(),
            }
        }
    }
    impl PartialEq for Cave {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
        }
    }
    impl PartialEq<str> for Cave {
        fn eq(&self, other: &str) -> bool {
            self.name == other
        }
    }
    impl std::hash::Hash for Cave {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.name.hash(state);
        }
    }

    pub struct CaveSystem {
        start: NodeIndex,
        graph: UnGraph<Cave, ()>,
    }
    impl FromStr for CaveSystem {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            struct RawPassage<'a> {
                cave1: &'a str,
                cave2: &'a str,
            }
            impl<'a> Parseable<'a> for RawPassage<'a> {
                fn parser(input: &'a str) -> NomParseResult<&str, Self> {
                    map(
                        separated_pair(alphanumeric1, tag("-"), alphanumeric1),
                        |(cave1, cave2)| Self { cave1, cave2 },
                    )(input)
                }
            }

            // Build the cave nodes and passages (edges).
            let mut graph = Graph::new_undirected();
            let mut caves = HashMap::new();
            for line in s.lines() {
                let raw_passage = RawPassage::from_str(line.trim())?;

                let index1 = *caves
                    .entry(raw_passage.cave1)
                    .or_insert_with(|| graph.add_node(Cave::new(raw_passage.cave1)));
                let index2 = *caves
                    .entry(raw_passage.cave2)
                    .or_insert_with(|| graph.add_node(Cave::new(raw_passage.cave2)));
                graph.add_edge(index1, index2, ());
            }

            Ok(Self {
                start: *caves.get("start").unwrap(),
                graph,
            })
        }
    }
    impl CaveSystem {
        pub fn paths(
            &self,
            special_cave: Option<NodeIndex>,
        ) -> Result<HashSet<Vec<&Cave>>, AocError> {
            // Remaining visits for each cave
            let mut visits_left = HashMap::new();
            for index in self.graph.node_indices() {
                let cave = self.graph.node_weight(index).unwrap();

                visits_left.insert(
                    index,
                    match cave.cave_type {
                        CaveType::Big => Infinitable::Infinity,
                        _ => Infinitable::Finite(if let Some(idx) = special_cave && idx == index { 2 } else { 1 }),
                    },
                );
            }

            // Paths from the given cave to the end having already visited some
            fn paths_rec<'a>(
                graph: &'a UnGraph<Cave, ()>,
                index: NodeIndex,
                visits_left: &HashMap<NodeIndex, Infinitable<usize>>,
            ) -> Option<HashSet<Vec<&'a Cave>>> {
                let mut paths = HashSet::new();
                let cave = graph.node_weight(index).unwrap();

                if cave.cave_type == CaveType::End {
                    // We've reached the end so this is the only path
                    paths.insert(vec![cave]);
                    Some(paths)
                } else {
                    //let x = visits_left.get(&index).unwrap() - Infinitable::Finite(1usize);
                    todo!()
                }
            }

            paths_rec(&self.graph, self.start, &visits_left).ok_or(AocError::NoSolution)
        }

        pub fn paths_special(&self) -> Result<HashSet<Vec<&Cave>>, AocError> {
            /* let mut paths = HashSet::new();
            for cave in self.caves.iter().filter(|c| c.has_type(CaveType::Small)) {
                paths.extend(self.paths(SpecialCave::Some {
                    cave: cave.clone(),
                    count: 2,
                })?)
            }
            Ok(paths) */
            todo!()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 12,
    name: "Passage Pathing",
    preprocessor: Some(|input| Ok(Box::new(CaveSystem::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<CaveSystem>()?
                    .paths(None)?
                    .len()
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            // Generation
            /* let cave_system = CaveSystem::from_str(input.expect_input()?)?;
            let paths = cave_system.paths_special()?;

            // Process
            Ok(Answer::Unsigned(paths.len().try_into().unwrap())) */
            todo!()
        },
    ],
};
