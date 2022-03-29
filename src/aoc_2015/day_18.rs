use std::marker::PhantomData;

use crate::aoc::prelude::*;
use cgmath::Vector2;

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
    fn stuck(_size: &(isize, isize), _point: &Vector2<isize>) -> bool {
        false
    }
}
#[derive(Clone)]
struct PartA;
impl Part for PartA {}
#[derive(Clone)]
struct PartB;
impl Part for PartB {
    fn stuck(size: &(isize, isize), point: &Vector2<isize>) -> bool {
        (point.x == 0 || point.x == size.0 - 1) && (point.y == 0 || point.y == size.1 - 1)
    }
}

#[derive(Clone, CharGridDebug)]
#[generics(PartB)]
struct LightGrid<P> {
    size: (usize, usize),
    data: Box<[Box<[bool]>]>,
    phant: PhantomData<P>,
}
impl<P: Part> Grid for LightGrid<P> {
    type Element = bool;

    fn size(&self) -> (usize, usize) {
        self.size
    }

    fn element_at(&mut self, point: &GridPoint) -> &mut Self::Element {
        &mut self.data[point.1][point.0]
    }
}
impl<P: Part> CharGrid for LightGrid<P> {
    fn default(size: (usize, usize)) -> Self {
        Self {
            size,
            data: vec![vec![false; size.0].into_boxed_slice()].into_boxed_slice(),
            phant: PhantomData::new(),
        }
    }

    fn from_char(c: char) -> Option<<Self as Grid>::Element> {
        match c {
            '#' => Some(true),
            '.' => Some(false),
            _ => None,
        }
    }

    fn to_char(e: &<Self as Grid>::Element) -> char {
        if *e {
            '#'
        } else {
            '.'
        }
    }

    /*
    TODO need to put this somewhere, GOD DAMMIT
    fn from_data(size: (usize, usize), data: Box<[Box<[Self::Element]>]>) -> AocResult<Self>
    where
        Self: Sized,
    {
        let mut grid = LightGrid {
            size: (size.0.try_into().unwrap(), size.1.try_into().unwrap()),
            data,
            phant: PhantomData {},
        };

        // Turn stuck lights on
        for point in grid.next_iter() {
            if P::stuck(&grid.size, &point) {
                let up = grid.point_usize(&point).unwrap();
                grid.data[up.y][up.x] = true;
            }
        }
        Ok(grid)
    }*/
}
impl<P: Part> Evolver<bool> for LightGrid<P> {
    type Point = GridPoint;

    fn new(other: &Self) -> Self {
        Self::default(other.size)
    }

    fn get(&self, point: &Self::Point) -> bool {
        self.get_element(point)
    }

    fn set(&mut self, point: &Self::Point, value: bool) {
        self.set_element(point, value)
    }

    fn next_cell(&self, point: &Self::Point) -> bool {
        let occupied: usize = self.neighbor_points(point, true, false).count();
        if self.get(point) {
            occupied == 2 || occupied == 3
        } else {
            occupied == 3
        }
    }

    fn next_iter(&self) -> Box<dyn Iterator<Item = Self::Point>> {
        self.all_points()
    }
}
impl<P: Part> LightGrid<P> {
    fn lights_on(&self) -> u64 {
        self.next_iter().filter_count(|point| self.get(point))
    }
}

fn solve<P: Part + Clone>(grid: &LightGrid<P>) -> AocResult<Answer> {
    Ok(grid.evolutions().nth(100 - 1).unwrap().lights_on().into())
}

pub const SOLUTION: Solution = Solution {
    day: 18,
    name: "Like a GIF For Your Yard",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let grid = LightGrid::<PartA>::from_str(input)?;

            // Process
            solve(&grid)
        },
        // Part b)
        |input| {
            // Generation
            let grid = LightGrid::<PartB>::from_str(input)?;

            /*for grid in grid.evolutions().take(5) {
                println!("{:?}", grid);
            }*/

            // Process
            solve(&grid)
        },
    ],
};
