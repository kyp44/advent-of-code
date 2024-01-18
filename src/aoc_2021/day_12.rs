use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "start-A
start-b
A-c
A-b
b-d
A-end
b-end";
            answers = unsigned![10, 36];
        }
        example {
            input = "dc-end
HN-start
start-kj
dc-start
dc-HN
LN-dc
HN-end
kj-sa
kj-HN
kj-dc";
            answers = unsigned![19, 103];
        }
        example {
            input = "fs-end
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
start-RW";
            answers = unsigned![226, 3509];
        }
        actual_answers = unsigned![4011, 108035];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::tree_search::{GlobalAction, GlobalState, GlobalStateTreeNode};
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
                        _ => Infinitable::Finite(
                            if let Some(idx) = special_cave
                                && idx == index
                            {
                                2
                            } else {
                                1
                            },
                        ),
                    },
                );
            }

            // Perform the tree search.
            PathTip {
                graph: &self.graph,
                tip: self.start,
                visits_left,
                path: vec![self.graph.node_weight(self.start).unwrap()],
            }
            .traverse_tree()
            .paths
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

    /// Global state used for the path tree search.
    #[derive(Debug, Default)]
    struct PathGlobalState<'a> {
        /// Set of complete paths through the cave from the start cave to the end cave.
        paths: HashSet<Vec<&'a Cave>>,
    }
    impl<'a> GlobalState<PathTip<'a>> for PathGlobalState<'a> {
        fn update_with_node(&mut self, node: &PathTip<'a>) {
            self.paths.insert(node.path.clone());
        }

        fn complete(&self) -> bool {
            // Never want the state to terminate recursion
            false
        }
    }

    /// The end of a path through the cave system, which is a node in the tree search.
    #[derive(Debug)]
    struct PathTip<'a> {
        /// The graph of the cave system.
        graph: &'a UnGraph<Cave, ()>,
        /// The graph node of the cave the ends the current path.
        tip: NodeIndex,
        /// Maps the cave graph node to the number of visits remaining for that cave.
        visits_left: HashMap<NodeIndex, Infinitable<usize>>,
        /// The path through the cave system, which includes the current cave as the
        /// last element.
        path: Vec<&'a Cave>,
    }
    impl<'a> GlobalStateTreeNode for PathTip<'a> {
        type GlobalState = PathGlobalState<'a>;

        fn recurse_action(
            &self,
            _state: &Self::GlobalState,
        ) -> aoc::tree_search::GlobalAction<Self> {
            let cave = self.graph.node_weight(self.tip).unwrap();

            if cave.cave_type == CaveType::End {
                // We've reached the end so add this path to the list.
                GlobalAction::Apply
            } else {
                let num_visits = *self.visits_left.get(&self.tip).unwrap();
                if num_visits > 0.into() {
                    // We can visit this cave again, so first mark that it was visited.
                    let mut visits_left = self.visits_left.clone();
                    *visits_left.get_mut(&self.tip).unwrap() = num_visits - 1.into();

                    // Now go through connecting caves and recurse
                    GlobalAction::Continue(
                        self.graph
                            .neighbors(self.tip)
                            .filter(|nc| {
                                self.graph.node_weight(*nc).unwrap().cave_type != CaveType::Start
                            })
                            .map(|next_cave| Self {
                                graph: self.graph,
                                tip: next_cave,
                                visits_left: visits_left.clone(),
                                path: {
                                    let mut path = self.path.clone();
                                    path.push(cave);
                                    path
                                },
                            })
                            .collect(),
                    )
                } else {
                    // Cannot visit this cave again so we're done
                    GlobalAction::Stop
                }
            }
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
