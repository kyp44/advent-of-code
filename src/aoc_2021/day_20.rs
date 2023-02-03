use std::{rc::Rc, str::FromStr};

use bitbuffer::{BitReadBuffer, BitWriteStream, LittleEndian};
use cgmath::Vector2;
use nom::{character::complete::one_of, combinator::map, multi::many_m_n};

use aoc::{parse::trim, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(5361), Unsigned(16826)],
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
    vec![35u64, 3351].answer_vec()
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

#[derive(Debug, Clone)]
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
            grid: Grid::from_str::<Grid<bool>>(sections[1].trim())?,
            infinity_pixels: false,
        })
    }
}
impl Image {
    fn next_size(&self) -> GridSize {
        self.grid.size() + GridSize::new(2, 2)
    }

    fn num_lit(&self) -> usize {
        self.grid.all_values().filter_count(|b| **b)
    }
}
impl Evolver<bool> for Image {
    type Point = Vector2<isize>;

    fn next_default(other: &Self) -> Self {
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
            grid: Grid::default(other.next_size()),
            infinity_pixels,
        }
    }

    fn get_element(&self, point: &Self::Point) -> bool {
        match self.grid.valid_point(point) {
            Some(p) => *self.grid.get(&p),
            None => self.infinity_pixels,
        }
    }

    fn set_element(&mut self, point: &Self::Point, value: bool) {
        self.grid.set(&point.try_point_into().unwrap(), value);
    }

    fn next_cell(&self, point: &Self::Point) -> bool {
        let mut binary_data = vec![];
        let mut write_stream = BitWriteStream::new(&mut binary_data, LittleEndian);

        // New grid is offset so need to convert point into current grid space
        let point = point - Self::Point::new(1, 1);
        let bits: Vec<bool> = self
            .grid
            .all_neighbor_points(point, true, true)
            .map(|p| self.get_element(&p))
            .collect();
        // The suggestion by the clippy lint does not compile.
        /*for b in self
            .grid
            .all_neighbor_points(point, true, true)
            .map(|p| self.get_element(&p))
            .rev()
        {*/
        for b in bits.into_iter().rev() {
            write_stream.write_bool(b).unwrap();
        }

        let read_buffer = BitReadBuffer::new(&binary_data, LittleEndian);
        let binary_value = read_buffer.read_int(0, read_buffer.bit_len()).unwrap();

        self.algorithm.lookup(binary_value).unwrap()
    }

    fn next_iter(&self) -> Box<dyn Iterator<Item = Self::Point>> {
        Box::new(
            self.next_size()
                .all_points()
                .map(|p| p.try_point_into().unwrap()),
        )
    }
}

pub const SOLUTION: Solution = Solution {
    day: 20,
    name: "Trench Map",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let image = Image::from_str(input.expect_input()?)?;

            // Process
            Ok(Answer::Unsigned(
                image
                    .evolutions()
                    .iterations(2)
                    .unwrap()
                    .num_lit()
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            // Generation
            let image = Image::from_str(input.expect_input()?)?;

            // Process
            Ok(Answer::Unsigned(
                image
                    .evolutions()
                    .iterations(50)
                    .unwrap()
                    .num_lit()
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
