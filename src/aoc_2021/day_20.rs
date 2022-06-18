use std::{rc::Rc, str::FromStr};

use cgmath::Vector2;
use nom::{character::complete::one_of, combinator::map, multi::many_m_n};

use crate::aoc::{parse::trim, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(123)],
    "..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##..##
#..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#......#.###
.######.###.####...#.##.##..#..#..#####.....#.#....###..#.##......#.....#.
.#..#..##..#...##.######.####.####.#.#...#.......#..#.#.#...####.##.#.....
.#..#...##.#.##..#...##.#.##..###.#......#.#.......#.#.#.####.###.##...#..
...####.#..#..#.##.#....##..#.####....##...##..#...#......#.#.......#.....
..##..####..#...#.#.#...##..#.#..###..#####........#..####......#..#

#..#.
#....
##..#
..#..
..###",
    vec![123u64].answer_vec()
    }
}

const ALG_SIZE: usize = 512;

#[derive(Clone, Debug)]
struct Algorithm {
    table: [bool; ALG_SIZE],
}
impl Parseable<'_> for Algorithm {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        map(
            many_m_n(
                ALG_SIZE,
                ALG_SIZE,
                map(trim(true, one_of(".#")), |c| c == '#'),
            ),
            |v| {
                let table = v.try_into().unwrap();
                Self { table }
            },
        )(input)
    }
}
impl Algorithm {
    fn lookup(&self, value: usize) -> Option<bool> {
        self.table.get(value).copied()
    }
}

#[derive(Debug)]
struct Image {
    algorithm: Rc<Algorithm>,
    grid: Grid<bool>,
    infinity_pixels: bool,
}
impl FromStr for Image {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sections = s.sections(2)?;
        let algorithm = Algorithm::from_str(sections[0].trim())?;

        Ok(Self {
            algorithm: Rc::new(algorithm),
            grid: Grid::grid_from_str(sections[1].trim())?,
            infinity_pixels: false,
        })
    }
}
impl Evolver<bool> for Image {
    type Point = Vector2<isize>;

    fn new(other: &Self) -> Self {
        let infinity_pixels = other
            .algorithm
            .lookup(if other.infinity_pixels {
                ALG_SIZE - 1
            } else {
                0
            })
            .unwrap();

        Self {
            algorithm: other.algorithm.clone(),
            grid: Grid::default(other.grid.size() + GridSize::new(2, 2)),
            infinity_pixels,
        }
    }

    fn get_element(&self, point: &Self::Point) -> bool {
        let size: Self::Point = self.grid.size().try_point_into().unwrap();
        if (0..size.x).contains(&point.x) && (0..size.y).contains(&point.y) {
            *self.grid.get(&point.try_point_into().unwrap())
        } else {
            self.infinity_pixels
        }
    }

    fn set_element(&mut self, point: &Self::Point, value: bool) {
        self.grid.set(&point.try_point_into().unwrap(), value);
    }

    fn next_cell(&self, point: &Self::Point) -> bool {
        todo!()
    }

    fn next_iter(&self) -> Box<dyn Iterator<Item = Self::Point>> {
        todo!()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 20,
    name: "Trench Map",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let image = Image::from_str(input.expect_input()?)?;

            println!("{:?}", image);

            // Process
            Ok(0u64.into())
        },
    ],
};
