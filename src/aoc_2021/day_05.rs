use std::{cmp::max, ops::RangeInclusive, str::FromStr};

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
    vec![Unsigned(5835)],
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
            LineIterator::new(LineType::Diagonal(
                range(self.from.x, self.to.x),
                range(self.from.y, self.to.y),
            ))
        }
    }
}
enum LineType {
    Horizontal(RangeInclusive<usize>, usize),
    Vertical(usize, RangeInclusive<usize>),
    Diagonal(RangeInclusive<usize>, RangeInclusive<usize>),
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
            LineType::Diagonal(xr, yr) => xr.next().zip(yr.next()),
        }
    }
}

#[derive(CharGridDebug)]
struct FloorMap {
    data: Box<[Box<[u8]>]>,
}
impl CharGrid for FloorMap {
    type Element = u8;

    fn default() -> Self::Element {
        0
    }

    fn from_char(c: char) -> Self::Element {
        if c == '.' {
            0
        } else {
            c.to_digit(10).unwrap().try_into().unwrap()
        }
    }

    fn to_char(e: &Self::Element) -> char {
        if *e == 0 {
            '.'
        } else {
            char::from_digit((*e).into(), 10).unwrap()
        }
    }

    fn from_data(_size: (usize, usize), data: Box<[Box<[Self::Element]>]>) -> AocResult<Self>
    where
        Self: Sized,
    {
        Ok(Self { data })
    }

    fn to_data(&self) -> &[Box<[Self::Element]>] {
        &self.data
    }
}
impl FloorMap {
    fn inc_location(&mut self, location: (usize, usize)) {
        self.data[location.1][location.0] += 1;
    }
}

trait Part {
    fn set_floor_map(lines: &[Line], floor_map: &mut FloorMap);
}
struct PartA {}
impl Part for PartA {
    fn set_floor_map(lines: &[Line], floor_map: &mut FloorMap) {
        for line in lines
            .iter()
            .filter(|l| l.from.x == l.to.x || l.from.y == l.to.y)
        {
            for location in line.iter() {
                floor_map.inc_location(location)
            }
        }
    }
}
struct PartB {}
impl Part for PartB {
    fn set_floor_map(lines: &[Line], floor_map: &mut FloorMap) {
        for line in lines.iter() {
            for location in line.iter() {
                floor_map.inc_location(location)
            }
        }
    }
}

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
        let mut floor_map = FloorMap::blank(size)?;

        // Now "draw" the lines on the map
        P::set_floor_map(&self.lines, &mut floor_map);

        Ok(floor_map)
    }

    fn num_overlap_points<P: Part>(&self) -> AocResult<usize> {
        Ok(self
            .floor_map::<P>()?
            .to_data()
            .iter()
            .flat_map(|row| row.iter())
            .filter(|v| **v > 1)
            .count())
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
            let map = vents.floor_map::<PartB>()?;
            println!("{:?}", map);

            // Process
            Ok(u64::try_from(vents.num_overlap_points::<PartB>()?)
                .unwrap()
                .into())
        },
    ],
};
