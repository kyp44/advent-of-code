use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "..##.......
#...#...#..
.#....#..#.
..#.#...#.#
.#...##..#.
..#.##.....
.#.#.#....#
.#........#
#.##...#...
#...##....#
.#..#...#.#
";
            answers = unsigned![7, 336];
        }
        actual_answers = unsigned![225, 1115775000];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::grid::{GridSpace, StdBool};
    use derive_new::new;
    use euclid::Vector2D;

    /// Map denoting open squares and trees, which can be parsed from text input.
    pub struct Map {
        /// Boolean grid for the hill denoting where trees are located.
        grid: Grid<StdBool>,
    }
    impl From<Grid<StdBool>> for Map {
        fn from(value: Grid<StdBool>) -> Self {
            Self { grid: value }
        }
    }
    impl Map {
        /// Returns whether a certain point on the map is a tree or not.
        fn is_tree(&self, point: &GridPoint) -> bool {
            let x = point.x % self.grid.size().width;
            **self.grid.get(&GridPoint::new(x, point.y))
        }
    }

    /// An [`Iterator`] for over whether the points taken on a downhill route through
    /// a [`Map`] with a particular slope have a tree or not.
    #[derive(new)]
    struct MapDownhill<'a> {
        /// Map through which we are traversing.
        map: &'a Map,
        /// Slope down the hill.
        slope: Vector2D<usize, GridSpace>,
        /// Current point on the map.
        #[new(value = "GridPoint::origin()")]
        point: GridPoint,
    }
    impl Iterator for MapDownhill<'_> {
        type Item = bool;

        fn next(&mut self) -> Option<Self::Item> {
            // If past the map vertically then we are done
            if self.point.y >= self.map.grid.size().height {
                return None;
            }

            // Get current position
            let tree = self.map.is_tree(&self.point);

            // Ready the next position
            self.point += self.slope;

            Some(tree)
        }
    }

    /// Counts the number of trees encountered on the way down for a particular [`Map`] and slope.
    pub fn count_slope(map: &Map, slope: Vector2D<usize, GridSpace>) -> u64 {
        MapDownhill::new(map, slope).filter_count(|t| *t)
    }
}

use euclid::vec2;
use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 3,
    name: "Toboggan Trajectory",
    preprocessor: Some(|input| Ok(Box::new(Map::from_grid_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(count_slope(input.expect_data::<Map>()?, vec2(3, 1)).into())
        },
        // Part two
        |input| {
            // Process
            let map = input.expect_data::<Map>()?;
            let slopes: [(usize, usize); 5] = [(1, 1), (3, 1), (5, 1), (7, 1), (1, 2)];
            Ok(slopes
                .iter()
                .map(|(x, y)| count_slope(map, vec2(*x, *y)))
                .product::<u64>()
                .into())
        },
    ],
};
