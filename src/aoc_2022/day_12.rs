use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi";
            answers = unsigned![31, 29];
        }
        actual_answers = unsigned![440, 439];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use petgraph::{algo::dijkstra, graph::NodeIndex, Graph};

    /// A square in in the height map.
    #[derive(Clone)]
    enum Square {
        /// The designated start space.
        Start,
        /// The designated end space.
        End,
        /// A normal space with its height as a lowercase ASCII character.
        Height(char),
    }
    impl TryFrom<char> for Square {
        type Error = ();

        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                'S' => Ok(Self::Start),
                'E' => Ok(Self::End),
                _ => {
                    if value.is_ascii_lowercase() {
                        Ok(Self::Height(value))
                    } else {
                        Err(())
                    }
                }
            }
        }
    }
    impl Square {
        /// The numerical height of the square, with `a` being zero and `z` being 25.
        ///
        /// Note that the start space has height 0 and the end space has height 25.
        pub fn height(&self) -> u8 {
            let c = match self {
                Square::Start => 'a',
                Square::End => 'z',
                Square::Height(c) => *c,
            };

            c.as_ascii().unwrap().to_u8() - 97
        }
    }

    /// The entire height map.
    pub struct HeightMap {
        /// The graph representing the height map.
        ///
        /// The nodes are the spaces and the edges represent directed paths between adjacent spaces.
        graph: Graph<Square, ()>,
        /// The node index for the designated start space.
        start: NodeIndex,
        /// The node index for the designated end space.
        end: NodeIndex,
    }
    impl FromStr for HeightMap {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // Parse grid and determine start and end points
            let grid = Grid::from_str(s)?;

            let start = grid
                .all_points()
                .find(|p| matches!(grid.get(p), Square::Start))
                .ok_or(AocError::InvalidInput("No start cell!".into()))?;
            let end = grid
                .all_points()
                .find(|p| matches!(grid.get(p), Square::End))
                .ok_or(AocError::InvalidInput("No end cell!".into()))?;

            // Transform the height map into a graph.
            let (graph, node_grid) =
                grid.as_graph(false, |p, np| (np.height() <= p.height() + 1).then_some(()));

            Ok(Self {
                graph,
                start: *node_grid.get(&start),
                end: *node_grid.get(&end),
            })
        }
    }
    impl HeightMap {
        /// Uses Dijkstra’s shortest path algorithm to determine the shortest path length from the
        /// `start` square to the designated end square.
        ///
        /// The shortest path length is returned, or [None] if there is no path from `start` to the end.
        fn fewest_steps(&self, start: NodeIndex) -> Option<u64> {
            let map = dijkstra(&self.graph, start, Some(self.end), |_| 1);

            map.get(&self.end).copied()
        }

        /// Determines the shortest path length from the designated start space to the designated
        /// end space.
        ///
        /// Returns an error if there is no complete path at all.
        pub fn fewest_steps_from_start(&self) -> AocResult<u64> {
            self.fewest_steps(self.start).ok_or(AocError::NoSolution)
        }

        /// Determines the shortest among all the shortest path lengths from every lowest square,
        /// that is squares with a height of 0 (`a` in the original map ).
        ///
        /// Note that this of course includes the designated start square.
        /// Returns an error if none of the lowest squares have a complete path to the end square at all.
        pub fn fewest_steps_from_lowest(&self) -> AocResult<u64> {
            self.graph
                .node_indices()
                .filter_map(|ni| {
                    if self.graph.node_weight(ni).unwrap().height() == 0 {
                        self.fewest_steps(ni)
                    } else {
                        None
                    }
                })
                .min()
                .ok_or(AocError::NoSolution)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 12,
    name: "Hill Climbing Algorithm",
    preprocessor: Some(|input| Ok(Box::new(HeightMap::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<HeightMap>()?
                .fewest_steps_from_start()?
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<HeightMap>()?
                .fewest_steps_from_lowest()?
                .into())
        },
    ],
};
