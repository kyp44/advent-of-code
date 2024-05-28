use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "0,9 -> 5,9
8,0 -> 0,8
9,4 -> 3,4
2,2 -> 2,1
7,0 -> 7,4
6,4 -> 2,0
0,9 -> 2,9
3,4 -> 1,4
0,0 -> 8,8
5,5 -> 8,2";
            answers = unsigned![5, 12];
        }
        actual_answers = unsigned![5835, 17013];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{grid::Digit, parse::separated};
    use derive_new::new;
    use nom::{
        bytes::complete::tag,
        character::complete::multispace0,
        combinator::map,
        sequence::{delimited, separated_pair},
    };
    use std::{cmp::max, iter::Rev, ops::RangeInclusive};

    /// A closed 2D line segment of a hydrothermal vent, which can be parsed
    /// from text input.
    pub struct Line {
        /// One end of the segment.
        from: GridPoint,
        /// The other end of the segment.
        to: GridPoint,
    }
    impl Parsable<'_> for Line {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            /// This is an internal function of [`Line::parser`], which is a [`nom`] parser for
            /// a single point on the 2D grid.
            fn point_parser(input: &str) -> NomParseResult<&str, GridPoint> {
                map(
                    separated_pair(
                        nom::character::complete::u16,
                        delimited(multispace0, tag(","), multispace0),
                        nom::character::complete::u16,
                    ),
                    |(x, y)| GridPoint::new(x.into(), y.into()),
                )(input)
            }
            map(
                separated_pair(point_parser, separated(tag("->")), point_parser),
                |(from, to)| Line { from, to },
            )(input)
        }
    }
    impl Line {
        /// Returns the type for this line.
        fn line_type(&self) -> LineType {
            /// This is an internal function of [`Line::line_type`], that simply returns an inclusive
            /// range from the smallest of two integers to the largest.
            fn range(a: usize, b: usize) -> RangeInclusive<usize> {
                a.min(b)..=a.max(b)
            }

            if self.from.y == self.to.y {
                LineType::Horizontal(range(self.from.x, self.to.x), self.from.y)
            } else if self.from.x == self.to.x {
                LineType::Vertical(self.from.x, range(self.from.y, self.to.y))
            } else {
                let diagonal_down = LineType::DiagonalDown(
                    range(self.from.x, self.to.x),
                    range(self.from.y, self.to.y),
                );
                let diagonal_up = LineType::DiagonalUp(
                    range(self.from.x, self.to.x),
                    range(self.from.y, self.to.y).rev(),
                );
                if self.from.x < self.to.x {
                    if self.from.y < self.to.y {
                        diagonal_down
                    } else {
                        diagonal_up
                    }
                } else if self.from.y < self.to.y {
                    diagonal_up
                } else {
                    diagonal_down
                }
            }
        }

        /// Returns a [`LineIterator`] for this line.
        fn iter(&self) -> LineIterator {
            LineIterator::new(self.line_type())
        }
    }

    /// The type of a 2D line segment.
    enum LineType {
        /// A strictly horziontal line segment with the range of `x` and `y`.
        Horizontal(RangeInclusive<usize>, usize),
        /// A strictly vertical line segment with `x` and the range of `y`.
        Vertical(usize, RangeInclusive<usize>),
        /// Diagonally down going from left to right with the ranges of `x` and `y`.
        DiagonalDown(RangeInclusive<usize>, RangeInclusive<usize>),
        /// Diagonally up going from left to right with the ranges of `x` and `y`.
        DiagonalUp(RangeInclusive<usize>, Rev<RangeInclusive<usize>>),
    }

    /// An [`Iterator`] over the integer points along a 2D line segment, including the
    /// end points.
    #[derive(new)]
    struct LineIterator {
        /// The type of the line that includes the range iterators.
        line_type: LineType,
    }
    impl Iterator for LineIterator {
        type Item = GridPoint;

        fn next(&mut self) -> Option<Self::Item> {
            match &mut self.line_type {
                LineType::Horizontal(xr, y) => xr.next().map(|x| GridPoint::new(x, *y)),
                LineType::Vertical(x, yr) => yr.next().map(|y| GridPoint::new(*x, y)),
                LineType::DiagonalDown(xr, yr) => {
                    xr.next().zip(yr.next()).map(|(x, y)| GridPoint::new(x, y))
                }
                LineType::DiagonalUp(xr, yr) => {
                    xr.next().zip(yr.next()).map(|(x, y)| GridPoint::new(x, y))
                }
            }
        }
    }

    /// Map of the ocean floor.
    pub struct FloorMap {
        /// The grid of the number of lines that cover each integer point.
        grid: Grid<Digit>,
    }
    impl From<Grid<Digit>> for FloorMap {
        fn from(value: Grid<Digit>) -> Self {
            Self { grid: value }
        }
    }
    impl FloorMap {
        /// Increments the number for a point.
        fn increment_point(&mut self, point: &GridPoint) {
            *self.grid.get_mut(point) += 1.into();
        }

        /// Counts the number of integer points that have more than one vent
        /// overlapping there.
        pub fn num_overlap_points(&self) -> u64 {
            self.grid.all_values().filter_count(|v| **v > 1.into())
        }
    }

    /// Behavior specific to one particular part of the problem.
    pub trait Part {
        /// Returns whether a [`Line`] should be used for this part.
        fn line_filter(_line: &Line) -> bool;
    }

    /// Behavior for part one, which includes only horizontal and vertical lines.
    pub struct PartOne {}
    impl Part for PartOne {
        fn line_filter(line: &Line) -> bool {
            // Keep only vertical or horizontal lines.
            matches!(
                line.line_type(),
                LineType::Horizontal(_, _) | LineType::Vertical(_, _)
            )
        }
    }

    /// Behavior for part two, which includes all lines.
    pub struct PartTwo {}
    impl Part for PartTwo {
        fn line_filter(_line: &Line) -> bool {
            true
        }
    }

    /// Locations of all of the vents, which can be parsed from text input.
    pub struct Vents {
        /// The set of 2D vent lines.
        lines: Box<[Line]>,
    }
    impl FromStr for Vents {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                lines: Line::gather(s.lines())?.into_boxed_slice(),
            })
        }
    }
    impl Vents {
        /// Creates the [`FloorMap`] for these vents, filtering the vent lines
        /// according to the [`Part`].
        pub fn floor_map<P: Part>(&self) -> AocResult<FloorMap> {
            // First determine how large the grid needs to be
            let max = |f: fn(&GridPoint) -> usize| {
                self.lines
                    .iter()
                    .map(|l| max(f(&l.from), f(&l.to)))
                    .max()
                    .unwrap()
            };
            let size = GridSize::new(max(|p| p.x) + 1, max(|p| p.y) + 1);

            // Create blank map
            let mut floor_map = FloorMap::default(size);

            // Now "draw" the lines on the map
            for line in self.lines.iter().filter(|line| P::line_filter(line)) {
                for loc in line.iter() {
                    floor_map.increment_point(&loc)
                }
            }

            Ok(floor_map)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 5,
    name: "Hydrothermal Venture",
    preprocessor: Some(|input| Ok(Box::new(Vents::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Vents>()?
                .floor_map::<PartOne>()?
                .num_overlap_points()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Vents>()?
                .floor_map::<PartTwo>()?
                .num_overlap_points()
                .into())
        },
    ],
};
