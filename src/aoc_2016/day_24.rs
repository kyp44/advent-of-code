use aoc::{prelude::*, tree_search::BestCostTreeNode};

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "###########
#0.1.....2#
#.#######.#
#4.......3#
###########";
            answers = unsigned![14, 20];
        }
        actual_answers = unsigned![498, 804];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::collections::HashMap;

    use super::*;
    use aoc::tree_search::{ApplyNodeAction, BestCostChild, BestCostTreeNode, Steps};
    use petgraph::{
        algo::dijkstra,
        graph::{NodeIndex, UnGraph},
    };

    /// An element of the input [`Grid`].
    #[derive(Clone, Copy, Debug)]
    enum MapGridElement {
        /// An open space.
        Open,
        /// A wall.
        Wall,
        /// A waypoint, with the robot starting at waypoint 0.
        Waypoint(u8),
    }
    impl TryFrom<char> for MapGridElement {
        type Error = ();

        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                '.' => Ok(Self::Open),
                '#' => Ok(Self::Wall),
                _ => value
                    .to_digit(10)
                    .map(|d| Self::Waypoint(d.try_into().unwrap()))
                    .ok_or(()),
            }
        }
    }
    impl MapGridElement {
        /// Returns whether an element is open an can be passed through.
        ///
        /// Every element is open except walls.
        pub fn is_open(&self) -> bool {
            !matches!(self, Self::Wall)
        }

        /// Returns the waypoint number if this is a waypoint and `None`
        /// otherwise.
        pub fn waypoint_number(&self) -> Option<u8> {
            match self {
                MapGridElement::Waypoint(w) => Some(*w),
                _ => None,
            }
        }
    }

    /// A reduced representation of the duct map.
    ///
    /// Can be parsed from text input.
    #[derive(Debug)]
    pub struct DuctMap {
        /// The reduced graph.
        ///
        /// Each node is a waypoint and includes the starting robot location
        /// (waypoint 0) with the waypoint number as its weight. Every node is
        /// connected to every other with the edge weight being the minimal
        /// numbers of steps between them.
        graph: UnGraph<u8, usize>,
        /// The index of the initial robot location node (waypoint 0).
        robot_idx: NodeIndex,
        /// The waypoint node indices, not including waypoint 0.
        waypoint_idxs: Vec<NodeIndex>,
    }
    impl FromStr for DuctMap {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // Parse the grid
            let grid = Grid::<MapGridElement>::from_str(s)?;

            // Transform into a graph
            let (graph, _) =
                grid.as_graph(false, |a, b| (a.is_open() && b.is_open()).then_some(()));

            // Find all waypoint node indices (including the robot waypoint 0)
            let waypoint_idxs: Vec<_> = graph
                .node_indices()
                .filter(|idx| graph.node_weight(*idx).unwrap().waypoint_number().is_some())
                .collect();

            // Verify that there is exactly one robot location (waypoint 0)
            if waypoint_idxs.iter().filter_count::<usize>(|idx| {
                matches!(graph.node_weight(**idx).unwrap().waypoint_number(), Some(0))
            }) != 1
            {
                return Err(AocError::InvalidInput(
                    "There must be exactly one robot location".into(),
                ));
            }

            // Create reduced graph
            let mut ungraph = UnGraph::new_undirected();

            // Add waypoint nodes while building map from original waypoint indices to
            // reduced indices
            let idx_map = HashMap::<_, _>::from_iter(waypoint_idxs.into_iter().map(|idx| {
                (
                    idx,
                    ungraph.add_node(graph.node_weight(idx).unwrap().waypoint_number().unwrap()),
                )
            }));

            // Add reduced edges
            for (idx_a, ridx_a) in idx_map.iter() {
                let distance_map = dijkstra(&graph, *idx_a, None, |_| 1usize);
                for idx_b in idx_map.keys() {
                    if idx_b != idx_a {
                        ungraph.update_edge(
                            *ridx_a,
                            *idx_map.get(idx_b).unwrap(),
                            *distance_map.get(idx_b).unwrap(),
                        );
                    }
                }
            }

            Ok(Self {
                robot_idx: *idx_map
                    .values()
                    .find(|ridx| *ungraph.node_weight(**ridx).unwrap() == 0)
                    .unwrap(),
                waypoint_idxs: idx_map
                    .values()
                    .copied()
                    .filter(|ridx| *ungraph.node_weight(*ridx).unwrap() != 0)
                    .collect(),
                graph: ungraph,
            })
        }
    }
    impl DuctMap {
        /// Returns the minimal distance between two waypoint nodes.
        fn minimal_distance(&self, idx_a: NodeIndex, idx_b: NodeIndex) -> usize {
            *self
                .graph
                .edges_connecting(idx_a, idx_b)
                .next()
                .unwrap()
                .weight()
        }

        /// Returns the waypoint tree search node for the initial state of the
        /// map.
        pub fn waypoint_search_node<P: Part>(&self) -> WaypointSearchNode<'_, P> {
            WaypointSearchNode {
                duct_map: self,
                current_idx: self.robot_idx,
                path: vec![],
                remaining_waypoints: self.waypoint_idxs.clone(),
                part: P::default(),
            }
        }
    }

    /// A part of the problem that allows a final waypoint once all the others
    /// have been visited.
    pub trait Part: Clone + Default {
        /// Returns the final waypoint if there is one.
        ///
        /// This is called by [`WaypointSearchNode`] once all
        /// other waypoints have been visited. Note that this will be called
        /// again once the provided waypoint has also been visited.
        fn final_waypoint(&mut self, duct_map: &DuctMap) -> Option<NodeIndex>;
    }

    /// The [`Part`] for part one, which provides no final waypoint.
    #[derive(Clone, Default)]
    pub struct PartOne;
    impl Part for PartOne {
        fn final_waypoint(&mut self, _duct_map: &DuctMap) -> Option<NodeIndex> {
            None
        }
    }

    /// The [`Part`] for part two, which provides the robot initial position
    /// once.
    #[derive(Clone, Default)]
    pub struct PartTwo {
        /// Whether the robot position has already been provided.
        already_provided: bool,
    }
    impl Part for PartTwo {
        fn final_waypoint(&mut self, duct_map: &DuctMap) -> Option<NodeIndex> {
            if self.already_provided {
                None
            } else {
                self.already_provided = true;
                Some(duct_map.robot_idx)
            }
        }
    }

    /// [`BestCostTreeNode`] for searching a [`DuctMap`] for the minimum number
    /// of steps for the robot to visit all required waypoints for a particular
    /// part of the problem `P`.
    #[derive(Clone)]
    pub struct WaypointSearchNode<'a, P: Part> {
        /// The duct map we are searching.
        duct_map: &'a DuctMap,
        /// The node index of the current location of the robot.
        current_idx: NodeIndex,
        /// The path of nodes taken to get to this point.
        path: Vec<NodeIndex>,
        /// The remaining waypoint nodes still left to visit.
        remaining_waypoints: Vec<NodeIndex>,
        /// The [`Part`] of the problem being solved.
        part: P,
    }
    impl<P: Part> PartialEq for WaypointSearchNode<'_, P> {
        fn eq(&self, other: &Self) -> bool {
            std::ptr::eq(self.duct_map, other.duct_map)
                && self.current_idx == other.current_idx
                && self.path == other.path
                && self.remaining_waypoints == other.remaining_waypoints
        }
    }
    impl<P: Part> Eq for WaypointSearchNode<'_, P> {}
    impl<P: Part> std::hash::Hash for WaypointSearchNode<'_, P> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            std::ptr::hash(self.duct_map, state);
            self.current_idx.hash(state);
            self.path.hash(state);
            self.remaining_waypoints.hash(state);
        }
    }
    impl<P: Part> BestCostTreeNode for WaypointSearchNode<'_, P> {
        type Metric = Steps;
        type NodeData = ();

        fn recurse_action(&mut self) -> ApplyNodeAction<BestCostChild<Self>, Self::NodeData> {
            // Have we visited every waypoint?
            if self.remaining_waypoints.is_empty() {
                match self.part.final_waypoint(self.duct_map) {
                    Some(idx) => self.remaining_waypoints.push(idx),
                    None => return ApplyNodeAction::Stop(Some(())),
                }
            }

            // Find all waypoints with this minimum distance (will usually just be one)
            ApplyNodeAction::Continue(
                self.remaining_waypoints
                    .iter()
                    .map(|next_idx| {
                        let mut remaining_waypoints = self.remaining_waypoints.clone();
                        remaining_waypoints.retain(|i| i != next_idx);

                        let mut path = self.path.clone();
                        path.push(*next_idx);

                        BestCostChild::new(
                            Self {
                                duct_map: self.duct_map,
                                current_idx: *next_idx,
                                path,
                                remaining_waypoints,
                                part: self.part.clone(),
                            },
                            self.duct_map
                                .minimal_distance(self.current_idx, *next_idx)
                                .into(),
                        )
                    })
                    .collect(),
            )
        }
    }
}

use solution::*;

/// Processes the solution for a [`Part`] of the problem.
fn process<P: Part>(input: &SolverInput) -> AocResult<Answer> {
    let minimal_steps: usize = input
        .expect_data::<DuctMap>()?
        .waypoint_search_node::<P>()
        .traverse_tree()?
        .cost
        .into();

    Ok(u64::try_from(minimal_steps).unwrap().into())
}

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 24,
    name: "Air Duct Spelunking",
    preprocessor: Some(|input| Ok(Box::new(DuctMap::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| process::<PartOne>(input),
        // Part two
        |input| process::<PartTwo>(input),
    ],
};
