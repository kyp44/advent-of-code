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
    map: Grid<u8>,
}
impl CharGrid<u8> for FloorMap {
    fn from_char(c: char) -> Option<u8> {
        c.to_digit(10).map(|v| v.try_into().unwrap())
    }

    fn to_char(e: &u8) -> char {
        char::from_digit((*e).into(), 10).unwrap()
    }
}
impl FloorMap {
    fn low_points(&self) -> impl Iterator<Item = GridPoint> + '_ {
        self.all_points().filter_map(|point| {
            let height = self.get(&point);
            if self
                .neighbor_points(&point, false, false)
                .all(|p| height < self.get(&p))
            {
                Some(point)
            } else {
                None
            }
        })
    }

    fn low_heights(&self) -> impl Iterator<Item = u8> + '_ {
        self.low_points().map(|p| *self.get(&p))
    }

    fn basin_region_size(&self, point: GridSize, points: &mut HashSet<GridPoint>) -> u64 {
        // Base cases
        if *self.get(&point) == 9 || points.contains(&point) {
            return 0;
        }

        let mut reg_size = 1;
        points.insert(point);
        if point.x > 0 {
            reg_size += self.basin_region_size(point - GridPoint::unit_x(), points);
        }
        if point.x < self.size.x - 1 {
            reg_size += self.basin_region_size(point + GridPoint::unit_x(), points);
        }
        if point.y > 0 {
            reg_size += self.basin_region_size(point - GridPoint::unit_y(), points);
        }
        if point.y < self.size.y - 1 {
            reg_size += self.basin_region_size(point + GridPoint::unit_y(), points);
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
                .map(|p| floor_map.basin_region_size(p, &mut HashSet::new()))
                .sorted()
                .rev()
                .take(3)
                .product::<u64>()
                .into())
        },
    ],
};
