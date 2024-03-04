use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "1163751742
1381373672
2136511328
3694931569
7463417111
1319128137
1359912421
3125421639
1293138521
2311944581";
            answers = unsigned![40, 315];
        }
        actual_answers = unsigned![398, 2817];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::grid::Digit;
    use bare_metal_modulo::{MNum, OffsetNumC};
    use derive_more::{Add, Deref};
    use euclid::Vector2D;
    use petgraph::{
        algo::dijkstra,
        graph::{DiGraph, NodeIndex},
    };

    /// A risk level, which is a single digit with modular arithmetic.
    ///
    /// The modulo arithmetic is needed when building the actual full grid in part two.
    #[derive(Clone, Copy, Deref, Add)]
    pub struct RiskLevel(OffsetNumC<u8, 9, 1>);
    impl TryFrom<char> for RiskLevel {
        type Error = ();

        fn try_from(value: char) -> Result<Self, Self::Error> {
            Ok((*Digit::try_from(value)?).into())
        }
    }
    impl From<u8> for RiskLevel {
        fn from(value: u8) -> Self {
            Self(OffsetNumC::new(value))
        }
    }
    impl From<RiskLevel> for u64 {
        fn from(value: RiskLevel) -> Self {
            value.a().into()
        }
    }

    /// The risk level grid, which can be parsed from text input.
    pub struct RiskLevels {
        /// The grid of risk levels.
        grid: Grid<RiskLevel>,
        /// The grid of the graph nodes.
        node_grid: Grid<NodeIndex>,
        /// The directed graph.
        graph: DiGraph<RiskLevel, u64>,
    }
    impl From<Grid<RiskLevel>> for RiskLevels {
        fn from(value: Grid<RiskLevel>) -> Self {
            let (graph, node_grid) = value.as_graph(false, |_, d| Some(u64::from(*d)));

            Self {
                grid: value,
                node_grid,
                graph,
            }
        }
    }
    impl RiskLevels {
        /// Uses [Dijkstra's algorithm](https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm)
        /// to find the path with minimal total risk and returns the total minimal risk.
        pub fn min_risk(&self) -> u64 {
            let end = *self
                .node_grid
                .get(&(self.node_grid.size().to_vector() - Vector2D::new(1, 1)).to_point());

            let map = dijkstra(
                &self.graph,
                *self.node_grid.get(&GridPoint::origin()),
                Some(end),
                |e| *e.weight(),
            );

            *map.get(&end).unwrap()
        }

        /// Expands this map as a tile into a `n` by `n` tile area and each tile
        /// adds one to the risk levels of the tile above or to the left of it.
        pub fn full_map(&self, n: u8) -> Self {
            let mut base_rows: Vec<Box<[RiskLevel]>> = Vec::new();

            // First add all the additional columns for the first major row
            for row in self.grid.rows_iter() {
                base_rows.push(
                    (0..n)
                        .flat_map(|i| row.iter().map(move |r| *r + i.into()))
                        .collect(),
                );
            }

            // Now duplicate all the rows
            let mut rows = Vec::new();
            for i in 0..n {
                for row in base_rows.iter() {
                    rows.push(row.iter().map(|r| *r + i.into()).collect())
                }
            }

            Self::from(Grid::from_data(rows).unwrap())
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 15,
    name: "Chiton",
    preprocessor: Some(|input| Ok(Box::new(RiskLevels::from_grid_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<RiskLevels>()?.min_risk().into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<RiskLevels>()?
                .full_map(5)
                .min_risk()
                .into())
        },
    ],
};
