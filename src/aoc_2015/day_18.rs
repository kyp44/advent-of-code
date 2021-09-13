use std::marker::PhantomData;

use crate::aoc::prelude::*;
use cgmath::{Vector2, Zero};
use itertools::iproduct;

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
    size: (isize, isize),
    data: Box<[Box<[bool]>]>,
    phant: PhantomData<P>,
}
impl<P: Part> CharGrid for LightGrid<P> {
    type Element = bool;

    fn default() -> Self::Element {
        false
    }

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
        let mut grid = LightGrid {
            size: (size.0.try_into().unwrap(), size.1.try_into().unwrap()),
            data,
            phant: PhantomData {},
        };

        // Trun stuck lights on
        for point in grid.next_iter() {
            if P::stuck(&grid.size, &point) {
                let up = grid.point_usize(&point).unwrap();
                grid.data[up.y][up.x] = true;
            }
        }
        Ok(grid)
    }

    fn to_data(&self) -> &[Box<[Self::Element]>] {
        &self.data
    }
}
impl<P: Part> Evolver<bool> for LightGrid<P> {
    type Point = Vector2<isize>;
    type Iter = impl Iterator<Item = Self::Point>;

    fn new(other: &Self) -> Self {
        let size = (
            other.size.0.try_into().unwrap(),
            other.size.1.try_into().unwrap(),
        );
        let data =
            vec![vec![Self::default(); size.0].into_boxed_slice(); size.1].into_boxed_slice();
        Self::from_data(size, data).unwrap()
    }

    fn get(&self, point: &Self::Point) -> bool {
        match self.point_usize(point) {
            None => false,
            Some(p) => self.data[p.y][p.x],
        }
    }

    fn set(&mut self, point: &Self::Point, value: bool) {
        if !P::stuck(&self.size, point) {
            let up = self.point_usize(point).unwrap();
            self.data[up.y][up.x] = value;
        }
    }

    fn next_cell(&self, point: &Self::Point) -> bool {
        let occupied: usize = iproduct!(-1isize..=1, -1isize..=1)
            .map(|(dx, dy)| Vector2::new(dx, dy))
            .filter(|dp| *dp != Vector2::zero())
            .filter_count(|dp| self.get(&(point + dp)));
        if self.get(point) {
            occupied == 2 || occupied == 3
        } else {
            occupied == 3
        }
    }

    fn next_iter(&self) -> Self::Iter {
        iproduct!(0..self.size.0, 0..self.size.1).map(|(x, y)| Vector2::new(x, y))
    }
}
impl<P: Part> LightGrid<P> {
    fn point_usize(&self, point: &Vector2<isize>) -> Option<Vector2<usize>> {
        if (0..self.size.0).contains(&point.x) && (0..self.size.1).contains(&point.y) {
            return Some(Vector2::new(
                point.x.try_into().unwrap(),
                point.y.try_into().unwrap(),
            ));
        }
        None
    }

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
