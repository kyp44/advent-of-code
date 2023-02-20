use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
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

    /// The type of a cave, which can be parsed from text input.
    #[derive(PartialEq, Eq, Hash, Copy, Clone)]
    enum CaveType {
        /// The starting cave.
        Start,
        /// The ending cave we want to get to.
        End,
        /// A big cave that can be visited any number of times.
        Big,
        /// A small cave than can only be visited once (part one) or maybe twice (part two).
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

    /// A cave.
    #[derive(Eq)]
    pub struct Cave {
        /// The cave name.
        name: String,
        /// The type of the cave.
        cave_type: CaveType,
    }
    impl Debug for Cave {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.name)
        }
    }
    impl Cave {
        /// Creates a cave from a string.
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

    /// The entire cave system, which can be parsed from text input.
    pub struct CaveSystem {
        /// The graph node of the starting cave.
        start: NodeIndex,
        /// The graph of the cave system.
        graph: UnGraph<Cave, ()>,
    }
    impl FromStr for CaveSystem {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            /// Sub-struct used when parsing a [`CaveSystem`], which is a passage
            /// between caves and can be parsed from text input.
            struct RawPassage<'a> {
                /// Cave name at one end of the passage.
                cave1: &'a str,
                /// Cave name at the other end of passage.
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
        /// Determines and returns the set of all possible paths through the cave system,
        /// only ever visiting small caves at most once.
        pub fn paths(&self, special_cave: Option<NodeIndex>) -> HashSet<Vec<&Cave>> {
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

            /// Recursive sub-function of [`CaveSystem::paths`].
            ///
            /// Given the cave system graph, a current cave node, and the remaining visits
            /// for every cave, returns the set of possible paths from the current cave
            /// to the ending cave.
            fn paths_rec<'a>(
                graph: &'a UnGraph<Cave, ()>,
                index: NodeIndex,
                visits_left: &HashMap<NodeIndex, Infinitable<usize>>,
            ) -> HashSet<Vec<&'a Cave>> {
                let mut paths = HashSet::new();
                let cave = graph.node_weight(index).unwrap();

                if cave.cave_type == CaveType::End {
                    // We've reached the end so this is the only path
                    paths.insert(vec![cave]);
                } else {
                    let num_visits = *visits_left.get(&index).unwrap();
                    if num_visits > 0.into() {
                        // We can visit this cave again, so first mark that it was visited.
                        let mut visits_left = visits_left.clone();
                        *visits_left.get_mut(&index).unwrap() = num_visits - 1.into();

                        // Now go through connecting caves and recurse
                        for next_cave in graph.neighbors(index).filter(|nc| {
                            graph.node_weight(*nc).unwrap().cave_type != CaveType::Start
                        }) {
                            for mut path in paths_rec(graph, next_cave, &visits_left).into_iter() {
                                // Prepend current cave to path and add the path.
                                path.insert(0, cave);
                                paths.insert(path);
                            }
                        }
                    }
                }

                paths
            }

            paths_rec(&self.graph, self.start, &visits_left)
        }

        /// Determines and returns the set of all possible paths through the cave system,
        /// only ever visiting small caves at most once except for a single small cave, which
        /// may be visited twice.
        pub fn paths_special(&self) -> HashSet<Vec<&Cave>> {
            let mut paths = HashSet::new();
            for special_cave in self
                .graph
                .node_indices()
                .filter(|ni| self.graph.node_weight(*ni).unwrap().cave_type == CaveType::Small)
            {
                paths.extend(self.paths(Some(special_cave)));
            }

            paths
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
                    .paths(None)
                    .len()
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<CaveSystem>()?
                    .paths_special()
                    .len()
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
