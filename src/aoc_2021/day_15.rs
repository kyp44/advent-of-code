use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(398), Unsigned(2817)],
    "1163751742
1381373672
2136511328
3694931569
7463417111
1319128137
1359912421
3125421639
1293138521
2311944581",
    vec![40u64, 315].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use cgmath::Zero;
    use priority_queue::PriorityQueue;
    use std::{cmp::Reverse, collections::HashMap};

    /// The risk level grid, which can be parsed from text input.
    pub struct RiskLevels {
        /// The grid of risk levels.
        grid: Grid<u8>,
    }
    impl FromStr for RiskLevels {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                grid: Grid::from_str::<Grid<u8>>(s)?,
            })
        }
    }
    impl RiskLevels {
        /// Implements [Dijkstra's algorithm](https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm)
        /// to find the path with minimal total risk and returns the total minimal risk.
        pub fn min_risk(&self) -> u64 {
            let mut visited = HashMap::new();
            let mut queue = PriorityQueue::new();

            // Add starting point to queue
            queue.push(GridPoint::zero(), Reverse(0));

            // Destination point
            let dest = self.grid.size() - GridPoint::new(1, 1);

            // Fan out the visited nodes
            loop {
                let (current, dist) = queue.pop().unwrap();
                for neighbor in self.grid.neighbor_points(&current, false, false) {
                    let alt_dist = dist.0 + u64::from(*self.grid.get(&neighbor));
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
            let mut base_rows: Vec<Box<[u8]>> = Vec::new();

            /// Sub-function of [`RiskLevels::full_map`] that adds two risk level, wrapping around
            /// if the sum is greater than 9.
            ///
            /// TODO: This can be implemented as the [`std::ops::Add`] trait on the wrapper type once [`Grid`] is refactored.
            fn add_wrap(a: u8, b: u8) -> u8 {
                let s = a + b;
                if s > 9 {
                    (s % 10) + 1
                } else {
                    s
                }
            }

            // First add all the additional columns for the first major row
            for row in self.grid.rows_iter() {
                base_rows.push(
                    (0..n)
                        .flat_map(|i| row.iter().map(move |r| add_wrap(*r, i)))
                        .collect(),
                );
            }

            // Now duplicate all the rows
            let mut rows: Vec<Box<[u8]>> = Vec::new();
            for i in 0..n {
                for row in base_rows.iter() {
                    rows.push(row.iter().map(|r| add_wrap(*r, i)).collect())
                }
            }

            Self {
                grid: Grid::from_data(rows.into_boxed_slice()).unwrap(),
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 15,
    name: "Chiton",
    preprocessor: Some(|input| Ok(Box::new(RiskLevels::from_str(input)?).into())),
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
