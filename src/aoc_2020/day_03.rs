use crate::aoc::prelude::*;
use cgmath::Zero;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(225), Unsigned(1115775000)],
        "..##.......
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
",
        vec![7u64, 336].answer_vec()
    }
}

impl CharGrid<bool> for BasicGrid<bool> {
    fn from_char(c: char) -> Option<bool> {
        match c {
            '#' => Some(true),
            '.' => Some(false),
            _ => None,
        }
    }

    fn to_char(e: &bool) -> char {
        if *e {
            '#'
        } else {
            '.'
        }
    }
}

impl BasicGrid<bool> {
    fn is_tree(&self, point: &GridPoint) -> bool {
        *self.get(point)
    }
}

#[derive(new)]
struct MapDownhill<'a> {
    map: &'a BasicGrid<bool>,
    slope: GridPoint,
    #[new(value = "GridPoint::zero()")]
    point: GridPoint,
}

impl Iterator for MapDownhill<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        // If past the map vertically then we are done
        if self.point.y >= self.map.size().y {
            return None;
        }

        // Get current position
        let tree = self.map.is_tree(&self.point);

        // Ready the next position
        self.point += self.slope;

        Some(tree)
    }
}

fn count_slope(map: &BasicGrid<bool>, slope: GridPoint) -> u64 {
    MapDownhill::new(map, slope).filter_count(|t| *t)
}

pub const SOLUTION: Solution = Solution {
    day: 3,
    name: "Toboggan Trajectory",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let map = BasicGrid::from_str(input)?;

            // Process
            Ok(count_slope(&map, GridPoint::new(3, 1)).into())
        },
        // Part b)
        |input| {
            // Generation
            let map = BasicGrid::from_str(input)?;

            // Process
            let slopes: [(usize, usize); 5] = [(1, 1), (3, 1), (5, 1), (7, 1), (1, 2)];
            Ok(slopes
                .iter()
                .map(|(x, y)| count_slope(&map, GridPoint::new(*x, *y)))
                .product::<u64>()
                .into())
        },
    ],
};
