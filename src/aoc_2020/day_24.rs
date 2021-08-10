use cgmath::{Vector2, Zero};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map},
    multi::many1,
};
use std::{collections::HashSet, convert::TryInto, str::FromStr};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expensive_test, solution_test};
    use Answer::Number;

    solution_test! {
    vec![Number(354)],
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
    vec![10].answer_vec()
    }
}

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
impl From<Direction> for Vector2<i32> {
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
    fn follow(&self, start: Vector2<i32>) -> Vector2<i32> {
        self.directions
            .iter()
            .fold(start, |a, b| a + Vector2::<i32>::from(*b))
    }
}

struct Floor {
    routes: Box<[Route]>,
}
impl FromStr for Floor {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Floor {
            routes: Route::gather(s.lines())?.into_boxed_slice(),
        })
    }
}
impl Floor {
    fn flip_all(&self) -> HashSet<Vector2<i32>> {
        let mut blacks = HashSet::new();

        for route in self.routes.iter() {
            let tile = route.follow(Vector2::zero());
            if blacks.contains(&tile) {
                blacks.remove(&tile);
            } else {
                blacks.insert(tile);
            }
        }

        blacks
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
            Ok(Answer::Number(floor.flip_all().len().try_into().unwrap()))
        },
    ],
};
