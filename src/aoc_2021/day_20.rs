use aoc::prelude::*;
use std::str::FromStr;

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

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{grid::StdBool, parse::trim};
    use bitbuffer::{BitReadBuffer, BitWriteStream, LittleEndian};
    use cgmath::{Point2, Vector2};
    use nom::{character::complete::one_of, combinator::map, multi::many_m_n};
    use std::rc::Rc;

    /// The size of the image enhancement algorithm table.
    const ALG_SIZE: usize = 512;

    /// The image enhancement algorithm table, which can be parsed from text input.
    #[derive(Clone, Debug)]
    struct Algorithm {
        /// The table of enhanced pixel values.
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
        /// Looks up a pixel value in the table based on the binary number.
        fn lookup(&self, value: usize) -> Option<bool> {
            self.table.get(value).copied()
        }
    }

    /// An image that can be enhanced, which can be parsed from text input.
    #[derive(Debug, Clone)]
    pub struct Image {
        /// The enhancement algorithm table.
        algorithm: Rc<Algorithm>,
        /// The image grid of pixels.
        grid: Grid<StdBool>,
        /// The pixel value of all pixels outside the defined image grid space.
        infinity_pixels: bool,
    }
    impl FromStr for Image {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let sections = s.sections(2)?;
            let algorithm = Algorithm::from_str(sections[0].trim())?;

            Ok(Self {
                algorithm: Rc::new(algorithm),
                grid: Grid::from_str(sections[1].trim())?,
                infinity_pixels: false,
            })
        }
    }
    impl Image {
        /// Returns the size of the next enhanced image, which will be a bit larger.
        fn enhanced_size(&self) -> GridSize {
            self.grid.size() + GridSize::new(2, 2)
        }

        /// Counts the number of lit pixels in the image.
        pub fn num_lit(&self) -> usize {
            self.grid.all_values().filter_count(|b| ***b)
        }

        /// Returns the pixel value for a point on the image.
        fn get_pixel(&self, point: &Point2<isize>) -> bool {
            match self.grid.valid_point(point) {
                Some(p) => **self.grid.get(&p),
                None => self.infinity_pixels,
            }
        }
    }
    impl Evolver<bool> for Image {
        type Point = Point2<isize>;

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
                grid: Grid::default(other.enhanced_size()),
                infinity_pixels,
            }
        }

        fn set_element(&mut self, point: &Self::Point, value: bool) {
            self.grid
                .set(&GridPoint::try_point_from(*point).unwrap(), value.into());
        }

        fn next_cell(&self, point: &Self::Point) -> bool {
            let mut binary_data = vec![];
            let mut write_stream = BitWriteStream::new(&mut binary_data, LittleEndian);

            // New grid is offset so need to convert point into current grid space
            let point = point - Vector2::new(1, 1);
            let bits: Vec<bool> = self
                .grid
                .all_neighbor_points(point, true, true)
                .map(|p| self.get_pixel(&p))
                .collect();
            for b in bits.into_iter().rev() {
                write_stream.write_bool(b).unwrap();
            }

            let read_buffer = BitReadBuffer::new(&binary_data, LittleEndian);
            let binary_value = read_buffer.read_int(0, read_buffer.bit_len()).unwrap();

            self.algorithm.lookup(binary_value).unwrap()
        }

        fn next_iter(&self) -> Box<dyn Iterator<Item = Self::Point>> {
            Box::new(
                self.enhanced_size()
                    .all_points()
                    .map(|p| p.try_point_into().unwrap()),
            )
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 20,
    name: "Trench Map",
    preprocessor: Some(|input| Ok(Box::new(Image::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<Image>()?
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
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<Image>()?
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
