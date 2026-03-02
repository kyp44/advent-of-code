use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "start: 1,1
end: 7,4
designer: 10
max steps: 15";
            answers = unsigned![11, 21];
        }
        actual_answers = unsigned![82, 138];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::collections::HashMap;

    use super::*;
    use aoc::parse::trim;
    use euclid::Rect;
    use nom::{
        bytes::complete::tag,
        combinator::map,
        sequence::{preceded, separated_pair},
    };
    use petgraph::graph::{NodeIndex, UnGraph};

    /// The parameters that define the cubicle maze and our goals within it.
    ///
    /// Can be parsed from text input.
    pub struct Parameters {
        /// The starting coordinates.
        start: GridPoint,
        /// The ending coordinates.
        end: GridPoint,
        /// The designer's favorite number, used to determine where walls are.
        designer: usize,
        /// The max number of steps used in part two.
        max_steps: usize,
        /// The bounds of the grid space.
        ///
        /// This is bounded to prevent searching for a path to the end forever,
        /// but also large enough to ensure that we get contain all spaces
        /// reachable by `max_steps` steps.
        bounds: Rect<usize, GridSpace>,
    }
    impl Parsable for Parameters {
        type Parsed<'a> = Self;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            use nom::character::complete::usize as pusize;

            fn grid_point_parser(input: &str) -> NomParseResult<&str, GridPoint> {
                map(separated_pair(pusize, tag(","), pusize), |(x, y)| {
                    GridPoint::new(x, y)
                })
                .parse(input)
            }

            map(
                (
                    trim(true, preceded(tag("start: "), grid_point_parser)),
                    trim(true, preceded(tag("end: "), grid_point_parser)),
                    trim(true, preceded(tag("designer: "), pusize)),
                    trim(true, preceded(tag("max steps: "), pusize)),
                ),
                |(start, end, designer, max_steps)| {
                    const END_MULTIPLE: usize = 3;

                    // This ensures that we can find all space within the maximum number of steps
                    let search_size = GridSize::new(
                        start.x + max_steps.max(END_MULTIPLE * end.x),
                        start.y + max_steps.max(END_MULTIPLE * end.y),
                    );

                    Self {
                        start,
                        end,
                        designer,
                        max_steps,
                        bounds: Rect::from_size(search_size),
                    }
                },
            )
            .parse(input)
        }
    }
    impl Parameters {
        /// Returns whether given coordinates are a wall or not.
        fn is_wall(&self, point: &GridPoint) -> bool {
            let x = point.x;
            let y = point.y;

            let s = x * x + 3 * x + 2 * x * y + y + y * y + self.designer;
            s.count_ones() % 2 == 1
        }

        /// Creates the graph representation of the maze based on the
        /// parameters, calculates the minimum number of steps to get to every
        /// reachable space, and returns the [`CubeMaze`].
        pub fn cube_maze(&self) -> AocResult<CubeMaze> {
            // Build the graph recursively
            fn recurse_node(
                nodes: &mut HashMap<GridPoint, NodeIndex>,
                graph: &mut UnGraph<(), ()>,
                params: &Parameters,
                point: GridPoint,
            ) -> NodeIndex {
                // If this point is already a node then we are done
                if let Some(node_index) = nodes.get(&point) {
                    return *node_index;
                }

                // Add the node
                let node_index = graph.add_node(());
                nodes.insert(point, node_index);

                // If this is the end node there is no need to recurse
                if point == params.end {
                    return node_index;
                }

                // Now recurse to neighbors that are not walls
                for neighbor_point in point
                    .try_cast::<isize>()
                    .unwrap()
                    .all_neighbor_points(false, false)
                {
                    // Do not go off the grid in the negative direction
                    if neighbor_point.x < 0 || neighbor_point.y < 0 {
                        continue;
                    }

                    let neighbor_point = neighbor_point.try_cast().unwrap();

                    // This keeps things bounded
                    if !params.bounds.contains(neighbor_point) {
                        continue;
                    }

                    // We do not care about walls
                    if params.is_wall(&neighbor_point) {
                        continue;
                    }
                    let neighbor_idx = recurse_node(nodes, graph, params, neighbor_point);
                    graph.update_edge(node_index, neighbor_idx, ());
                }

                node_index
            }

            let mut nodes = HashMap::new();
            let mut graph = UnGraph::default();

            let start = recurse_node(&mut nodes, &mut graph, self, self.start);
            let end = *nodes.get(&self.end).ok_or(AocError::NoSolution)?;

            // `petgraph` uses `hashbrown::HashMap` but does not re-export `hashbrown`, so
            // we choose to convert it rather than add the extra dependency and try to keep
            // them in sync.
            let path_steps =
                HashMap::from_iter(petgraph::algo::dijkstra(&graph, start, Some(end), |_| 1u64));

            Ok(CubeMaze {
                end,
                max_steps: self.max_steps.try_into().unwrap(),
                path_steps,
            })
        }
    }

    /// The fully processed cube maze where the minimum number of steps to each
    /// reachable space is known.
    pub struct CubeMaze {
        /// The graph index of the ending node.
        end: NodeIndex,
        /// The max number of steps used in part two.
        max_steps: u64,
        /// Map of the graph node index to the minimum number of steps to reach
        /// the node.
        path_steps: HashMap<NodeIndex, u64>,
    }
    impl CubeMaze {
        /// Returns the minimum number of steps to the ending location.
        ///
        /// Fails if the ending location was not reachable.
        pub fn min_steps(&self) -> AocResult<u64> {
            self.path_steps
                .get(&self.end)
                .copied()
                .ok_or(AocError::NoSolution)
        }

        /// Returns the number of locations reachable from the maximum number of
        /// steps.
        pub fn reachable_locations(&self) -> u64 {
            self.path_steps
                .values()
                .filter(|ns| **ns <= self.max_steps)
                .count()
                .try_into()
                .unwrap()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 13,
    name: "A Maze of Twisty Little Cubicles",
    preprocessor: Some(|input| Ok(Box::new(Parameters::from_str(input)?.cube_maze()?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<CubeMaze>()?.min_steps()?.into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<CubeMaze>()?
                .reachable_locations()
                .into())
        },
    ],
};
