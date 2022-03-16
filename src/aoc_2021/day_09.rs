use std::collections::HashSet;

use itertools::{iproduct, Itertools};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
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

#[derive(CharGridDebug)]
struct FloorMap {
    size: (usize, usize),
    map: Box<[Box<[u8]>]>,
}
impl CharGrid for FloorMap {
    type Element = u8;

    fn default() -> Self::Element {
        0
    }

    fn from_char(c: char) -> Self::Element {
        c.to_digit(10).unwrap().try_into().unwrap()
    }

    fn to_char(e: &Self::Element) -> char {
        char::from_digit(u32::from(*e), 10).unwrap()
    }

    fn from_data(size: (usize, usize), data: Box<[Box<[Self::Element]>]>) -> AocResult<Self>
    where
        Self: Sized,
    {
        Ok(FloorMap { size, map: data })
    }

    fn to_data(&self) -> &[Box<[Self::Element]>] {
        &self.map
    }
}
impl FloorMap {
    fn neighbors(&self, x: usize, y: usize) -> impl Iterator<Item = u8> + '_ {
        iproduct!(-1..=1, -1..=1).filter_map(move |(dx, dy)| {
            let (x, y) = (
                isize::try_from(x).unwrap() + dx,
                isize::try_from(y).unwrap() + dy,
            );
            if x < 0 || y < 0 {
                return None;
            }
            let (x, y) = (usize::try_from(x).unwrap(), usize::try_from(y).unwrap());

            if x < self.size.0 && y < self.size.1 && ((dx == 0) ^ (dy == 0)) {
                Some(self.map[y][x])
            } else {
                None
            }
        })
    }

    fn low_points(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        iproduct!(0..self.size.1, 0..self.size.0).filter_map(|(y, x)| {
            let height = self.map[y][x];
            if self.neighbors(x, y).all(|h| height < h) {
                Some((x, y))
            } else {
                None
            }
        })
    }

    fn low_heights(&self) -> impl Iterator<Item = u8> + '_ {
        self.low_points().map(|(x, y)| self.map[y][x])
    }

    fn basin_region_size(&self, x: usize, y: usize, points: &mut HashSet<(usize, usize)>) -> u64 {
        // Base cases
        let point = (x, y);
        if self.map[y][x] == 9 || points.contains(&point) {
            return 0;
        }

        let mut reg_size = 1;
        points.insert(point);
        if x > 0 {
            reg_size += self.basin_region_size(x - 1, y, points);
        }
        if x < self.size.0 - 1 {
            reg_size += self.basin_region_size(x + 1, y, points);
        }
        if y > 0 {
            reg_size += self.basin_region_size(x, y - 1, points);
        }
        if y < self.size.1 - 1 {
            reg_size += self.basin_region_size(x, y + 1, points);
        }

        reg_size
    }
}

pub const SOLUTION: Solution = Solution {
    day: 9,
    name: "Smoke Basin",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let floor_map = FloorMap::from_str(input)?;

            // Process
            Ok(Answer::Unsigned(
                floor_map.low_heights().map(|h| u64::from(h) + 1).sum(),
            ))
        },
        // Part b)
        |input| {
            // Generation
            let floor_map = FloorMap::from_str(input)?;

            // Process
            Ok(floor_map
                .low_points()
                .map(|(x, y)| floor_map.basin_region_size(x, y, &mut HashSet::new()))
                .sorted()
                .rev()
                .take(3)
                .product::<u64>()
                .into())
        },
    ],
};
