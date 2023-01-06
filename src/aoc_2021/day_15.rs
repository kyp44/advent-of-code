use std::{cmp::Reverse, collections::HashMap, str::FromStr};

use cgmath::Zero;
use priority_queue::PriorityQueue;

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
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

struct RiskLevels {
    grid: Grid<u8>,
}
impl FromStr for RiskLevels {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            grid: Grid::grid_from_str(s)?,
        })
    }
}
impl RiskLevels {
    fn min_risk(&self) -> u64 {
        // Implements Dijkstra's algorithm to find the minimum path
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

    fn full_map(&self, num_copies: u8) -> Self {
        let mut base_rows: Vec<Box<[u8]>> = Vec::new();

        fn add_wrap(a: u8, b: u8) -> u8 {
            let s = a + b;
            if s > 9 {
                (s % 10) + 1
            } else {
                s
            }
        }

        // First add all the additional colums for the first major row
        for row in self.grid.rows_iter() {
            base_rows.push(
                (0..num_copies)
                    .flat_map(|i| row.iter().map(move |r| add_wrap(*r, i)))
                    .collect(),
            );
        }

        // Now duplicate all the rows
        let mut rows: Vec<Box<[u8]>> = Vec::new();
        for i in 0..num_copies {
            for row in base_rows.iter() {
                rows.push(row.iter().map(|r| add_wrap(*r, i)).collect())
            }
        }

        Self {
            grid: Grid::from_data(rows.into_boxed_slice()).unwrap(),
        }
    }
}

pub const SOLUTION: Solution = Solution {
    day: 15,
    name: "Chiton",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let risk_levels = RiskLevels::from_str(input.expect_input()?)?;

            // Process
            Ok(risk_levels.min_risk().into())
        },
        // Part two
        |input| {
            // Generation
            let risk_levels = RiskLevels::from_str(input.expect_input()?)?;

            // Process
            Ok(risk_levels.full_map(5).min_risk().into())
        },
    ],
};
