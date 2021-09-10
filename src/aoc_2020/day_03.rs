use crate::aoc::prelude::*;

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

struct Map {
    size: (usize, usize),
    data: Box<[Box<[bool]>]>,
}
impl CharGrid for Map {
    type Element = bool;

    fn from_char(c: char) -> Self::Element {
        c == '#'
    }

    fn to_char(e: &Self::Element) -> char {
        if *e {
            '#'
        } else {
            '.'
        }
    }

    fn from_data(size: (usize, usize), data: Box<[Box<[Self::Element]>]>) -> AocResult<Self>
    where
        Self: Sized,
    {
        Ok(Map { size, data })
    }

    fn to_data(&self) -> &[Box<[Self::Element]>] {
        &self.data
    }
}

impl Map {
    fn is_tree(&self, x: usize, y: usize) -> bool {
        self.data[y][x % self.size.0]
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
        if self.y >= self.map.size.1 {
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

fn count_slope(map: &Map, x: usize, y: usize) -> u64 {
    MapDownhill::new(map, x, y).filter_count(|t| *t)
}

pub const SOLUTION: Solution = Solution {
    day: 3,
    name: "Toboggan Trajectory",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let map = Map::from_str(input)?;

            // Process
            Ok(count_slope(&map, 3, 1).into())
        },
        // Part b)
        |input| {
            // Generation
            let map = Map::from_str(input)?;

            // Process
            let slopes: [(usize, usize); 5] = [(1, 1), (3, 1), (5, 1), (7, 1), (1, 2)];
            Ok(slopes
                .iter()
                .map(|(x, y)| count_slope(&map, *x, *y))
                .product::<u64>()
                .into())
        },
    ],
};
