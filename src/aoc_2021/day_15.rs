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
    use cgmath::{EuclideanSpace, Vector2};
    use derive_more::{Add, Deref, From, Into};
    use priority_queue::PriorityQueue;
    use std::{cmp::Reverse, collections::HashMap};

    /// A risk level, which is a single digit with modular arithmetic.
    #[derive(Clone, Copy, Deref, From, Into, Add)]
    pub struct RiskLevel(OffsetNumC<u8, 9, 1>);
    impl TryFrom<char> for RiskLevel {
        type Error = ();

        fn try_from(value: char) -> Result<Self, Self::Error> {
            Ok(OffsetNumC::new(*Digit::try_from(value)?).into())
        }
    }
    impl From<u8> for RiskLevel {
        fn from(value: u8) -> Self {
            OffsetNumC::new(value).into()
        }
    }

    /// The risk level grid, which can be parsed from text input.
    pub struct RiskLevels {
        /// The grid of risk levels.
        grid: Grid<RiskLevel>,
    }
    impl From<Grid<RiskLevel>> for RiskLevels {
        fn from(value: Grid<RiskLevel>) -> Self {
            Self { grid: value }
        }
    }
    impl RiskLevels {
        /// Implements [Dijkstra's algorithm](https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm)
        /// to find the path with minimal total risk and returns the total minimal risk.
        pub fn min_risk(&self) -> u64 {
            let mut visited = HashMap::new();
            let mut queue = PriorityQueue::new();

            // Add starting point to queue
            queue.push(PointExt::origin(), Reverse(0));

            // Destination point
            let dest = GridPoint::from_vec(self.grid.size() - Vector2::new(1, 1));

            // Fan out the visited nodes
            loop {
                let (current, dist) = queue.pop().unwrap();
                for neighbor in self.grid.neighbor_points(&current, false, false) {
                    let alt_dist = dist.0 + u64::from(self.grid.get(&neighbor).a());
                    match queue.get_priority(&neighbor) {
                        Some(d) => {
                            if alt_dist < d.0 {
                                queue.change_priority(&neighbor, Reverse(alt_dist));
                            }
                        }
                        None => {
                            queue.push(neighbor, Reverse(alt_dist));
                        }
                    }
                }
                if current == dest {
                    break dist.0;
                }
                visited.insert(current, dist);
            }
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

            Self {
                grid: Grid::from_data(rows).unwrap(),
            }
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
