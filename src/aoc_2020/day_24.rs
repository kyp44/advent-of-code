use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "sesenwnenenewseeswwswswwnenewsewsw
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
wseweeenwnesenwwwswnew";
            answers = unsigned![10, 2208];
        }
        actual_answers = unsigned![354, 3608];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use cgmath::{Point2, Vector2};
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

    /// Coordinates of a tile on the floor.
    ///
    /// Despite the [hexagonal tiling](https://en.wikipedia.org/wiki/Hexagonal_tiling),
    /// every tile can be specified with a 2D vector where the tiling is oriented so
    /// that horizontal lines are formed, along which the `x` coordinate varies.
    /// For a given tile, increasing the `y` coordinate, on the other hand,
    /// moves along a diagonal line to upper left so that decreasing the `y`
    /// coordinates moves to the lower right.
    type Point = Point2<i32>;

    /// Direction to go from a tile, which can be parsed from text input.
    ///
    /// Bear in mind that the tiling is [hexagonal](https://en.wikipedia.org/wiki/Hexagonal_tiling)
    /// and oriented such that horizontal lines of tiles are formed.
    #[derive(Debug, Copy, Clone, EnumIter)]
    enum Direction {
        /// The tile directly to the right.
        East,
        /// The tile directly to the left.
        West,
        /// The tile to the left and above.
        SouthEast,
        /// The tile to the left and below.
        SouthWest,
        /// The tile to the right and above.
        NorthEast,
        /// The tile to the right and below.
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

    /// A route to take on the tile floor, which can be parsed from text input.
    #[derive(Debug)]
    struct Route {
        /// Ordered list of directions to take from some starting tile.
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
        /// Follows the route given the starting tile [`Point`], and returns the
        /// tile [`Point`] on which you end up.
        fn follow(&self, start: Point) -> Point {
            self.directions
                .iter()
                .fold(start, |a, b| a + Vector2::<i32>::from(*b))
        }
    }

    /// A tile floor, which can be parsed from text input.
    ///
    /// The series of directions that are parsed are immediately followed to turn
    /// the requisite tiles black.
    #[derive(Clone)]
    pub struct Floor {
        /// Set of tile [`Point`]s that have been flipped over to be black.
        black_tiles: HashSet<Point>,
    }
    impl FromStr for Floor {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let routes = Route::gather(s.lines())?;

            // Determine the initial state
            let mut black_tiles = HashSet::new();
            for route in routes.iter() {
                let tile = route.follow(Point::origin());
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

        fn next_default(_other: &Self) -> Self {
            Floor {
                black_tiles: HashSet::new(),
            }
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
                .filter_count(|dp: &Vector2<i32>| self.black_tiles.contains(&(point + *dp)));
            if self.black_tiles.contains(point) {
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

            Box::new(iproduct!(range(|p| p.y), range(|p| p.x)).map(|(y, x)| Self::Point::new(x, y)))
        }
    }
    impl Floor {
        /// Counts the number of black tiles on the floor.
        pub fn num_black_tiles(&self) -> u64 {
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
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 24,
    name: "Lobby Layout",
    preprocessor: Some(|input| Ok(Box::new(input.parse::<Floor>()?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<Floor>()?.num_black_tiles().into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Floor>()?
                .evolutions()
                .iterations(100)
                .unwrap()
                .num_black_tiles()
                .into())
        },
    ],
};
