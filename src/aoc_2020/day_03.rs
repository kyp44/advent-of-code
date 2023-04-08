use aoc::prelude::*;
use cgmath::Vector2;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

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
            answers = vec![7u64, 336].answer_vec();
        }
        actual_answers = vec![Unsigned(225), Unsigned(1115775000)];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::grid::StdBool;
    use derive_new::new;

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
            let x = point.x % self.grid.size().x;
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
        slope: Vector2<usize>,
        /// Current point on the map.
        #[new(value = "GridPoint::new(0, 0)")]
        point: GridPoint,
    }
    impl Iterator for MapDownhill<'_> {
        type Item = bool;

        fn next(&mut self) -> Option<Self::Item> {
            // If past the map vertically then we are done
            if self.point.y >= self.map.grid.size().y {
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
    pub fn count_slope(map: &Map, slope: Vector2<usize>) -> u64 {
        MapDownhill::new(map, slope).filter_count(|t| *t)
    }
}

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
            Ok(count_slope(input.expect_data::<Map>()?, Vector2::new(3, 1)).into())
        },
        // Part two
        |input| {
            // Process
            let map = input.expect_data::<Map>()?;
            let slopes: [(usize, usize); 5] = [(1, 1), (3, 1), (5, 1), (7, 1), (1, 2)];
            Ok(slopes
                .iter()
                .map(|(x, y)| count_slope(map, Vector2::new(*x, *y)))
                .product::<u64>()
                .into())
        },
    ],
};
