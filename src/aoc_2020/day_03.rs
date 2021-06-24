use super::super::aoc::{AocError, FilterCount, Solution};
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![225, 1115775000],
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
        vec![7, 336]
    }
}

struct Map {
    width: usize,
    height: usize,
    data: Vec<Vec<bool>>,
}

impl Map {
    fn is_tree(&self, x: usize, y: usize) -> bool {
        self.data[y][x % self.width]
    }
}

impl FromStr for Map {
    type Err = AocError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut liter = s.lines();
        fn parse_row(line: &str) -> Vec<bool> {
            line.trim().chars().map(|c| c != '.').collect()
        }
        let first_row = parse_row(
            liter
                .next()
                .ok_or_else(|| AocError::InvalidInput("No lines".to_string()))?,
        );
        let width = first_row.len();
        if width < 1 {
            return Err(AocError::InvalidInput(
                "First map line has no content!".to_string(),
            ));
        }
        let mut data = vec![first_row];

        for line in liter {
            let row = parse_row(line);
            if row.len() != width {
                return Err(AocError::InvalidInput(format!(
                    "Map row '{}' has a length different from {}",
                    line, width
                )));
            }
            data.push(row);
        }
        Ok(Map {
            width,
            height: data.len(),
            data,
        })
    }
}

struct MapDownhill<'a> {
    map: &'a Map,
    dx: usize,
    dy: usize,
    x: usize,
    y: usize,
}

impl MapDownhill<'_> {
    fn new(map: &'_ Map, dx: usize, dy: usize) -> MapDownhill {
        MapDownhill {
            map,
            dx,
            dy,
            x: 0,
            y: 0,
        }
    }
}

impl Iterator for MapDownhill<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        // If past the map vertically then we are done
        if self.y >= self.map.height {
            return None;
        }

        // Get current position
        let tree = self.map.is_tree(self.x, self.y);

        // Ready the next position
        self.x += self.dx;
        self.y += self.dy;

        Some(tree)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 3,
    name: "Toboggan Trajectory",
    solver: |input| {
        // Generation
        let map = Map::from_str(input)?;

        // Process
        let count_slope = |x, y| MapDownhill::new(&map, x, y).filter_count(|t| *t);
        let slopes: [(usize, usize); 5] = [(1, 1), (3, 1), (5, 1), (7, 1), (1, 2)];
        let answers = vec![
            // Part a)
            count_slope(3, 1),
            // Part b)
            slopes.iter().map(|(x, y)| count_slope(*x, *y)).product(),
        ];

        Ok(answers)
    },
};
