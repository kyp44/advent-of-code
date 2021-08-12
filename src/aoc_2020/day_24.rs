use cgmath::{Vector2, Zero};
use itertools::{iproduct, Itertools};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map},
    multi::many1,
};
use std::{collections::HashSet, convert::TryInto, fmt, str::FromStr};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Number;

    solution_test! {
            vec![Number(354), Number(3608)],
        "sesenwnenenewseeswwswswwnenewsewsw
neeenesenwnwwswnenewnwwsewnenwseswesw
seswneswswsenwwnwse
nwnwneseeswswnenewneswwnewseswneseene
swweswneswnenwsewnwneneseenw
eesenwseswswnenwswnwnwsewwnwsene
sewnenenenesenwsewnenwwwse
wenwwweseeeweswwwnwwe
wsweesenenewnwwnwsenewsenwwsesesenwne
neeswseenwwswnwswswnw
nenwswwsewswnenenewsenwsenwnesesenew
enewnwewneswsewnwswenweswnenwsenwsw
sweneswneswneneenwnewenewwneswswnese
swwesenesewenwneswnwwneseswwne
enesenwswwswneneswsenwnewswseenwsese
wnwnesenesenenwwnenwsewesewsesesew
nenewswnwewswnenesenwnesewesw
eneswnwswnwsenenwnwnwwseeswneewsenese
neswnwewnwnwseenwseesewsenwsweewe
wseweeenwnesenwwwswnew",
    vec![10, 2208].answer_vec()
    }
}

type Point = Vector2<i32>;

#[derive(Debug, Copy, Clone)]
enum Direction {
    East,
    West,
    SouthEast,
    SouthWest,
    NorthEast,
    NorthWest,
}
impl Parseable<'_> for Direction {
    fn parser(input: &str) -> NomParseResult<Self> {
        use Direction::*;
        map(
            alt((
                tag("e"),
                tag("w"),
                tag("se"),
                tag("sw"),
                tag("ne"),
                tag("nw"),
            )),
            |s| match s {
                "e" => East,
                "w" => West,
                "se" => SouthEast,
                "sw" => SouthWest,
                "ne" => NorthEast,
                "nw" => NorthWest,
                _ => panic!(),
            },
        )(input)
    }
}
impl From<Direction> for Point {
    fn from(dir: Direction) -> Self {
        use Direction::*;
        match dir {
            East => Vector2::unit_x(),
            West => -Vector2::unit_x(),
            SouthEast => -Vector2::unit_y(),
            SouthWest => Vector2::new(-1, -1),
            NorthEast => Vector2::new(1, 1),
            NorthWest => Vector2::unit_y(),
        }
    }
}

#[derive(Debug)]
struct Route {
    directions: Box<[Direction]>,
}
impl Parseable<'_> for Route {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(all_consuming(many1(Direction::parser)), |vec| Route {
            directions: vec.into_boxed_slice(),
        })(input)
    }
}
impl Route {
    fn follow(&self, start: Point) -> Point {
        self.directions
            .iter()
            .fold(start, |a, b| a + Vector2::<i32>::from(*b))
    }
}

struct Floor {
    black_tiles: HashSet<Point>,
}
impl FromStr for Floor {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let routes = Route::gather(s.lines())?;

        // Determine the initial state
        let mut black_tiles = HashSet::new();
        for route in routes.iter() {
            let tile = route.follow(Vector2::zero());
            if black_tiles.contains(&tile) {
                black_tiles.remove(&tile);
            } else {
                black_tiles.insert(tile);
            }
        }

        Ok(Floor { black_tiles })
    }
}
impl Iterator for Floor {
    type Item = HashSet<Point>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut new_blacks = HashSet::new();
        for point in self.iter() {
            if self.black_tiles.contains(&point) {
                // Tile is black
                let adj = self.adjacent_blacks(&point);
                if adj > 0 && adj <= 2 {
                    new_blacks.insert(point);
                }
            } else {
                // Tile is white
                if self.adjacent_blacks(&point) == 2 {
                    new_blacks.insert(point);
                }
            }
        }
        self.black_tiles = new_blacks.clone();
        Some(new_blacks)
    }
}
impl fmt::Debug for Floor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // First convert to Vecs so that they can be reversed
        let mut rows: Vec<Vec<char>> = Vec::new();
        let mut row: Vec<char> = Vec::new();

        let mut last_y: Option<i32> = None;
        for point in self.iter() {
            if let Some(y) = last_y {
                if y != point.y {
                    rows.push(row);
                    row = Vec::new();
                }
            }
            last_y = Some(point.y);

            row.push(if self.black_tiles.contains(&point) {
                '#'
            } else {
                '.'
            });
        }
        rows.push(row);

        // Now output the Vecs
        for (i, row) in rows.into_iter().rev().enumerate() {
            write!(f, "{}", (0..i).map(|_| ' ').collect::<String>())?;
            writeln!(f, "{}", row.into_iter().join(" "))?;
        }

        Ok(())
    }
}
impl Floor {
    fn adjacent_blacks(&self, point: &Point) -> usize {
        [
            Point::new(0, 1),
            Point::new(0, -1),
            Point::new(-1, 0),
            Point::new(-1, -1),
            Point::new(1, 0),
            Point::new(1, 1),
        ]
        .iter()
        .filter_count(|dp| self.black_tiles.contains(&(*dp + point)))
    }

    fn iter(&self) -> impl Iterator<Item = Point> {
        // Determine the range in x and y
        let range = |f: fn(&Point) -> i32| {
            let start = self.black_tiles.iter().map(|p| f(p)).min().unwrap_or(1);
            let stop = self.black_tiles.iter().map(|p| f(p)).max().unwrap_or(-1);
            (start - 1)..=(stop + 1)
        };

        iproduct!(range(|p| p.y), range(|p| p.x)).map(|(y, x)| Vector2::new(x, y))
    }
}

pub const SOLUTION: Solution = Solution {
    day: 24,
    name: "Lobby Layout",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let floor: Floor = input.parse()?;

            // Process
            Ok(Answer::Number(floor.black_tiles.len().try_into().unwrap()))
        },
        // Part b)
        |input| {
            // Generation
            let mut floor: Floor = input.parse()?;

            // Process
            Ok(Answer::Number(
                floor.nth(100 - 1).unwrap().len().try_into().unwrap(),
            ))
        },
    ],
};
