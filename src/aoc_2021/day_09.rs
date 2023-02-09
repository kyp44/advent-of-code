use aoc::prelude::*;
use itertools::Itertools;
use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(512), Unsigned(1600104)],
    "2199943210
3987894921
9856789892
8767896789
9899965678",
    vec![15u64, 1134].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use aoc::grid::Digit;

    use super::*;

    /// Basin in the cave floor surrounding a low point.
    pub struct Basin<'a> {
        /// The floor map in which the basin resides.
        floor_map: &'a FloorMap,
        /// The low point in the map for this basin.
        low_point: GridPoint,
    }
    impl Basin<'_> {
        /// The height of the low point.
        pub fn low_height(&self) -> u8 {
            **self.floor_map.grid.get(&self.low_point)
        }

        /// The size of the basin.
        pub fn size(&self) -> u64 {
            /// Recursive subfunction of [`Basin::size`] that finds the size of a region
            /// given any point in the region.
            fn region_size(
                grid: &Grid<Digit>,
                point: GridPoint,
                points: &mut HashSet<GridPoint>,
            ) -> u64 {
                // Base cases
                if **grid.get(&point) == 9 || points.contains(&point) {
                    return 0;
                }

                let mut reg_size = 1;
                let size = grid.size();
                points.insert(point);
                if point.x > 0 {
                    reg_size += region_size(grid, point - GridPoint::unit_x(), points);
                }
                if point.x < size.x - 1 {
                    reg_size += region_size(grid, point + GridPoint::unit_x(), points);
                }
                if point.y > 0 {
                    reg_size += region_size(grid, point - GridPoint::unit_y(), points);
                }
                if point.y < size.y - 1 {
                    reg_size += region_size(grid, point + GridPoint::unit_y(), points);
                }

                reg_size
            }

            region_size(&self.floor_map.grid, self.low_point, &mut HashSet::new())
        }
    }

    /// Height map of the cave floor, which can parsed from text input.
    pub struct FloorMap {
        /// Grid of the heights.
        grid: Grid<Digit>,
    }
    impl From<Grid<Digit>> for FloorMap {
        fn from(value: Grid<Digit>) -> Self {
            Self { grid: value }
        }
    }
    impl FloorMap {
        /// Returns an [`Iterator`] over all of the basins on the cave floor.
        pub fn basins(&self) -> impl Iterator<Item = Basin> {
            self.grid
                .all_points()
                .filter(|point| {
                    let height = self.grid.get(point);
                    self.grid
                        .neighbor_points(point, false, false)
                        .all(|p| height < self.grid.get(&p))
                })
                .map(|low_point| Basin {
                    floor_map: self,
                    low_point,
                })
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 9,
    name: "Smoke Basin",
    preprocessor: Some(|input| Ok(Box::new(FloorMap::from_grid_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<FloorMap>()?
                .basins()
                .map(|basin| u64::from(basin.low_height() + 1))
                .sum::<u64>()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<FloorMap>()?
                .basins()
                .map(|basin| basin.size())
                .sorted()
                .rev()
                .take(3)
                .product::<u64>()
                .into())
        },
    ],
};
