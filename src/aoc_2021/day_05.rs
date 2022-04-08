use std::{cmp::max, iter::Rev, ops::RangeInclusive, str::FromStr};

use cgmath::Vector2;
use nom::{
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::map,
    sequence::{delimited, separated_pair},
};

use crate::aoc::{parse::separated, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(5835), Unsigned(17013)],
    "0,9 -> 5,9
8,0 -> 0,8
9,4 -> 3,4
2,2 -> 2,1
7,0 -> 7,4
6,4 -> 2,0
0,9 -> 2,9
3,4 -> 1,4
0,0 -> 8,8
5,5 -> 8,2",
    vec![5u64, 12].answer_vec()
    }
}

type Point = Vector2<usize>;

struct Line {
    from: Point,
    to: Point,
}
impl Parseable<'_> for Line {
    fn parser(input: &str) -> NomParseResult<Self> {
        fn point_parser(input: &str) -> NomParseResult<Point> {
            map(
                separated_pair(
                    nom::character::complete::u16,
                    delimited(multispace0, tag(","), multispace0),
                    nom::character::complete::u16,
                ),
                |(x, y)| Point::new(x.into(), y.into()),
            )(input)
        }
        map(
            separated_pair(point_parser, separated(tag("->")), point_parser),
            |(from, to)| Line { from, to },
        )(input)
    }
}
impl Line {
    fn iter(&self) -> LineIterator {
        fn range(a: usize, b: usize) -> RangeInclusive<usize> {
            if a < b {
                a..=b
            } else {
                b..=a
            }
        }

        if self.from.y == self.to.y {
            LineIterator::new(LineType::Horizontal(
                range(self.from.x, self.to.x),
                self.from.y,
            ))
        } else if self.from.x == self.to.x {
            LineIterator::new(LineType::Vertical(
                self.from.x,
                range(self.from.y, self.to.y),
            ))
        } else {
            let diagonal_down = LineIterator::new(LineType::DiagonalDown(
                range(self.from.x, self.to.x),
                range(self.from.y, self.to.y),
            ));
            let diagonal_up = LineIterator::new(LineType::DiagonalUp(
                range(self.from.x, self.to.x),
                range(self.from.y, self.to.y).rev(),
            ));
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
}
enum LineType {
    Horizontal(RangeInclusive<usize>, usize),
    Vertical(usize, RangeInclusive<usize>),
    DiagonalDown(RangeInclusive<usize>, RangeInclusive<usize>),
    DiagonalUp(RangeInclusive<usize>, Rev<RangeInclusive<usize>>),
}
struct LineIterator {
    line_type: LineType,
}
impl LineIterator {
    fn new(line_type: LineType) -> Self {
        LineIterator { line_type }
    }
}
impl Iterator for LineIterator {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.line_type {
            LineType::Horizontal(xr, y) => xr.next().map(|x| (x, *y)),
            LineType::Vertical(x, yr) => yr.next().map(|y| (*x, y)),
            LineType::DiagonalDown(xr, yr) => xr.next().zip(yr.next()),
            LineType::DiagonalUp(xr, yr) => xr.next().zip(yr.next()),
        }
    }
}

#[derive(CharGridDebug)]
struct FloorMap {
    grid: Grid<u8>,
}
impl CharGrid<u8> for FloorMap {
    fn from_char(c: char) -> Option<u8> {
        if c == '.' {
            Some(0)
        } else {
            c.to_digit(10).map(|v| v.try_into().unwrap())
        }
    }

    fn to_char(e: &u8) -> char {
        if *e == 0 {
            '.'
        } else {
            char::from_digit((*e).into(), 10).unwrap()
        }
    }
}
impl FloorMap {
    fn inc_location(&mut self, location: (usize, usize)) {
        self.data[location.1][location.0] += 1;
    }
}

trait Part {
    fn line_filter(_line: &&Line) -> bool {
        true
    }
}
struct PartA {}
impl Part for PartA {
    fn line_filter(line: &&Line) -> bool {
        line.from.x == line.to.x || line.from.y == line.to.y
    }
}
struct PartB {}
impl Part for PartB {}

struct Vents {
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
    fn floor_map<P: Part>(&self) -> AocResult<FloorMap> {
        todo!()
        /*
        // First determine how large the grid needs to be
        let max = |f: fn(&Point) -> usize| {
            self.lines
                .iter()
                .map(|l| max(f(&l.from), f(&l.to)))
                .max()
                .unwrap()
        };
        let size = (max(|p| p.x) + 1, max(|p| p.y) + 1);

        // Create blank map
        let mut floor_map = FloorMap::default(size);

        // Now "draw" the lines on the map
        for line in self.lines.iter().filter(P::line_filter) {
            for loc in line.iter() {
                floor_map.inc_location(loc)
            }
        }

        Ok(floor_map)*/
    }

    fn num_overlap_points<P: Part>(&self) -> AocResult<usize> {
        todo!()
        /*
        Ok(self
            .floor_map::<P>()?
            .to_data()
            .iter()
            .flat_map(|row| row.iter())
            .filter(|v| **v > 1)
            .count())*/
    }
}

pub const SOLUTION: Solution = Solution {
    day: 5,
    name: "Hydrothermal Venture",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let vents = Vents::from_str(input)?;

            // Process
            Ok(u64::try_from(vents.num_overlap_points::<PartA>()?)
                .unwrap()
                .into())
        },
        // Part b)
        |input| {
            // Generation
            let vents = Vents::from_str(input)?;

            // Process
            Ok(u64::try_from(vents.num_overlap_points::<PartB>()?)
                .unwrap()
                .into())
        },
    ],
};
