use cgmath::{Vector2, Zero};
use itertools::{iproduct, Itertools};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map},
    multi::many1,
};
use std::{collections::HashSet, convert::TryInto, fmt, str::FromStr};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
            vec![Unsigned(354), Unsigned(3608)],
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
    vec![10u64, 2208].answer_vec()
    }
}

type Point = Vector2<i32>;

#[derive(Debug, Copy, Clone, EnumIter)]
enum Direction {
    East,
    West,
    SouthEast,
    SouthWest,
    NorthEast,
    NorthWest,
}
impl Parseable<'_> for Direction {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
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
    fn parser(input: &str) -> NomParseResult<&str, Self> {
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

#[derive(Clone)]
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
impl Evolver<bool> for Floor {
    type Point = Point;

    fn new(_other: &Self) -> Self {
        Floor {
            black_tiles: HashSet::new(),
        }
    }

    fn get_element(&self, point: &Self::Point) -> bool {
        self.black_tiles.contains(point)
    }

    fn set_element(&mut self, point: &Self::Point, value: bool) {
        if value {
            self.black_tiles.insert(*point);
        } else {
            self.black_tiles.remove(point);
        }
    }

    fn next_cell(&self, point: &Self::Point) -> bool {
        let adj: usize = Direction::iter()
            .map(|d| d.into())
            .filter_count(|dp: &Point| self.get_element(&(*dp + point)));
        if self.get_element(point) {
            // Tile is black
            adj > 0 && adj <= 2
        } else {
            // Tile is white
            adj == 2
        }
    }

    fn next_iter(&self) -> Box<dyn Iterator<Item = Self::Point>> {
        // Determine the range in x and y
        let range = |f: fn(&Point) -> i32| match self.black_tiles.iter().map(f).range() {
            Some(r) => (r.start() - 1)..=(r.end() + 1),
            None => 0..=0,
        };

        Box::new(iproduct!(range(|p| p.y), range(|p| p.x)).map(|(y, x)| Vector2::new(x, y)))
    }
}
impl Floor {
    fn num_black_tiles(&self) -> u64 {
        self.black_tiles.len().try_into().unwrap()
    }
}
impl fmt::Debug for Floor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // First convert to Vecs so that they can be reversed
        let mut rows: Vec<Vec<char>> = Vec::new();
        let mut row: Vec<char> = Vec::new();

        let mut last_y: Option<i32> = None;
        for point in self.next_iter() {
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

pub const SOLUTION: Solution = Solution {
    day: 24,
    name: "Lobby Layout",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let floor: Floor = input.parse()?;

            //println!("{:?}", floor);

            // Process
            Ok(floor.num_black_tiles().into())
        },
        // Part b)
        |input| {
            // Generation
            let floor: Floor = input.parse()?;

            // Process
            Ok(floor
                .evolutions()
                .nth(100 - 1)
                .unwrap()
                .num_black_tiles()
                .into())
        },
    ],
};
