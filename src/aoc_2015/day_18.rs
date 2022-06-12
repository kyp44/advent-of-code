use std::{collections::HashSet, marker::PhantomData, str::FromStr};

use maplit::hashset;

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
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

trait Part {
    fn stuck_points(_grid: &Grid<bool>) -> HashSet<GridPoint> {
        // No stuck points by default
        HashSet::new()
    }
}
#[derive(Clone)]
struct PartA;
impl Part for PartA {}
#[derive(Clone)]
struct PartB;
impl Part for PartB {
    fn stuck_points(grid: &Grid<bool>) -> HashSet<GridPoint> {
        let size = grid.size();
        hashset![
            GridPoint::new(0, 0),
            GridPoint::new(size.x - 1, 0),
            GridPoint::new(0, size.y - 1),
            GridPoint::new(size.x - 1, size.y - 1),
        ]
    }
}

#[derive(Clone, CharGridDebug)]
#[generics(PartB)]
struct LightGrid<P> {
    grid: Grid<bool>,
    phant: PhantomData<P>,
}
impl<P: Part> CharGrid<bool> for LightGrid<P> {
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

    fn get_grid(&self) -> &Grid<bool> {
        &self.grid
    }
}
impl<P: Part> FromStr for LightGrid<P> {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut grid = Self::grid_from_str(s)?;
        for point in P::stuck_points(&grid) {
            grid.set(&point, true);
        }
        Ok(Self {
            grid,
            phant: PhantomData {},
        })
    }
}
impl<P: Part> Evolver<bool> for LightGrid<P> {
    type Point = GridPoint;

    fn new(other: &Self) -> Self {
        Self {
            grid: Grid::default(*other.grid.size()),
            phant: PhantomData {},
        }
    }

    fn get_element(&self, point: &Self::Point) -> bool {
        *self.grid.get(point)
    }

    fn set_element(&mut self, point: &Self::Point, value: bool) {
        self.grid.set(point, value)
    }

    fn next_cell(&self, point: &Self::Point) -> bool {
        if P::stuck_points(&self.grid).contains(point) {
            return true;
        }
        let occupied: usize = self
            .grid
            .neighbor_points(point, true, false)
            .filter_count(|p| self.get_element(p));
        if self.get_element(point) {
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
    fn lights_on(&self) -> u64 {
        self.next_iter().filter_count(|point| *self.grid.get(point))
    }
}

fn solve<P: Part + Clone>(grid: &LightGrid<P>) -> AocResult<Answer> {
    Ok(grid.evolutions().nth(100 - 1).unwrap().lights_on().into())
}

pub const SOLUTION: Solution = Solution {
    day: 18,
    name: "Like a GIF For Your Yard",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let grid = LightGrid::<PartA>::from_str(input.expect_input()?)?;

            // Process
            solve(&grid)
        },
        // Part b)
        |input| {
            // Generation
            let grid = LightGrid::<PartB>::from_str(input.expect_input()?)?;

            /*for grid in grid.evolutions().take(5) {
                println!("{:?}", grid);
            }*/

            // Process
            solve(&grid)
        },
    ],
};
