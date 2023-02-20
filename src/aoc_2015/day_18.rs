use std::str::FromStr;

use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(814), Unsigned(924)],
    ".#.#.#
...##.
#....#
..#...
#.#..#
####..",
    vec![4u64, 7].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::grid::StdBool;
    use maplit::hashset;
    use std::{collections::HashSet, marker::PhantomData};

    /// Behavior specific to one particular part of the problem.
    pub trait Part {
        /// Returns the set of lights that are stuck on given a grid.
        fn stuck_points(grid: &Grid<StdBool>) -> HashSet<GridPoint>;
    }

    /// Behavior for part one.
    #[derive(Clone)]
    pub struct PartOne;
    impl Part for PartOne {
        fn stuck_points(_grid: &Grid<StdBool>) -> HashSet<GridPoint> {
            HashSet::new()
        }
    }

    /// Behavior for part two.
    #[derive(Clone)]
    pub struct PartTwo;
    impl Part for PartTwo {
        fn stuck_points(grid: &Grid<StdBool>) -> HashSet<GridPoint> {
            let size = grid.size();
            hashset![
                GridPoint::new(0, 0),
                GridPoint::new(size.x - 1, 0),
                GridPoint::new(0, size.y - 1),
                GridPoint::new(size.x - 1, size.y - 1),
            ]
        }
    }

    /// The light grid evolver that can be parsed from text input.
    #[derive(Clone)]
    pub struct LightGrid<P> {
        /// The the actual grid.
        grid: Grid<StdBool>,
        /// Phantom data for the part.
        phant: PhantomData<P>,
    }
    impl<P: Part> FromStr for LightGrid<P> {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut grid = Grid::<StdBool>::from_str(s)?;
            for point in P::stuck_points(&grid) {
                grid.set(&point, true.into());
            }
            Ok(Self {
                grid,
                phant: PhantomData {},
            })
        }
    }
    impl<P: Part> Evolver<bool> for LightGrid<P> {
        type Point = GridPoint;

        fn next_default(other: &Self) -> Self {
            Self {
                grid: Grid::default(*other.grid.size()),
                phant: PhantomData {},
            }
        }

        fn set_element(&mut self, point: &Self::Point, value: bool) {
            self.grid.set(point, value.into())
        }

        fn next_cell(&self, point: &Self::Point) -> bool {
            if P::stuck_points(&self.grid).contains(point) {
                return true;
            }
            let occupied: usize = self
                .grid
                .neighbor_points(point, true, false)
                .filter_count(|p| **self.grid.get(p));
            if **self.grid.get(point) {
                occupied == 2 || occupied == 3
            } else {
                occupied == 3
            }
        }

        fn next_iter(&self) -> Box<dyn Iterator<Item = Self::Point>> {
            Box::new(self.grid.all_points())
        }
    }
    impl<P: Part> LightGrid<P> {
        /// Returns the number of lights that are on.
        fn lights_on(&self) -> u64 {
            self.next_iter()
                .filter_count(|point| **self.grid.get(point))
        }
    }

    /// Solves a part of the problem.
    pub fn solve<P: Part + Clone>(grid: &LightGrid<P>) -> AocResult<Answer> {
        Ok(grid
            .evolutions()
            .iterations(100)
            .unwrap()
            .lights_on()
            .into())
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 18,
    name: "Like a GIF For Your Yard",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let grid = LightGrid::<PartOne>::from_str(input.expect_input()?)?;

            // Process
            solve(&grid)
        },
        // Part two
        |input| {
            // Generation
            let grid = LightGrid::<PartTwo>::from_str(input.expect_input()?)?;

            /*for grid in grid.evolutions().take(5) {
                println!("{:?}", grid);
            }*/

            // Process
            solve(&grid)
        },
    ],
};
