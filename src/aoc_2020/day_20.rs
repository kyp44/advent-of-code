use std::str::FromStr;

use crate::aoc::{AocError, AocResult, DiscardInput, ParseError, Solution};
use nom::{
    bytes::complete::tag,
    character::complete::{digit1, line_ending, one_of},
    combinator::map,
    multi::{many1, separated_list1},
    sequence::{delimited, pair},
    Finish,
};

use enum_map::{enum_map, Enum, EnumMap};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![],
    "Tile 2311:
..##.#..#.
##..#.....
#...##..#.
####.#...#
##.##.###.
##...#.###
.#.#.#..##
..#....#..
###...#.#.
..###..###

Tile 1951:
#.##...##.
#.####...#
.....#..##
#...######
.##.#....#
.###.#####
###.##.##.
.###....#.
..#.#..#.#
#...##.#..

Tile 1171:
####...##.
#..##.#..#
##.#..#.#.
.###.####.
..###.####
.##....##.
.#...####.
#.##.####.
####..#...
.....##...

Tile 1427:
###.##.#..
.#..#.##..
.#.##.#..#
#.#.#.##.#
....#...##
...##..##.
...#.#####
.#.####.#.
..#..###.#
..##.#..#.

Tile 1489:
##.#.#....
..##...#..
.##..##...
..#...#...
#####...#.
#..#.#.#.#
...#.#.#..
##.#...##.
..##.##.##
###.##.#..

Tile 2473:
#....####.
#..#.##...
#.##..#...
######.#.#
.#...#.#.#
.#########
.###.#..#.
########.#
##...##.#.
..###.#.#.

Tile 2971:
..#.#....#
#...###...
#.#.###...
##.##..#..
.#####..##
.#..####.#
#..#.#..#.
..####.###
..#.#.###.
...#.#.#.#

Tile 2729:
...#.#.#.#
####.#....
..#.#.....
....#..#.#
.##..##.#.
.#.####...
####.#.#..
##.####...
##..#.##..
#.##...##.

Tile 3079:
#.#.#####.
.#..######
..#.......
######....
####.#..#.
.#...#.##.
#.#####.##
..#.###...
..#.......
..#.###...",
    vec![Some(20899048083289)]
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Enum)]
enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone)]
enum Rotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

#[derive(Debug)]
struct Tile {
    id: u64,
    edges: EnumMap<Edge, Vec<bool>>,
    edges_reversed: EnumMap<Edge, Vec<bool>>,
}
impl FromStr for Tile {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (id, lv) = map::<_, _, _, ParseError, _, _>(
            pair(
                delimited(tag("Tile "), digit1, pair(tag(":"), line_ending)),
                separated_list1(line_ending, many1(one_of(".#"))),
            ),
            |(ids, lv): (&str, Vec<Vec<char>>)| (ids.parse().unwrap(), lv),
        )(s)
        .finish()
        .discard_input()?;

        // Verify the tile dimensions
        let size = lv[0].len();
        if lv.len() != size {
            return Err(AocError::InvalidInput(format!(
                "Tile {} did not have the expected height of {}",
                id, size
            )));
        }
        if lv.iter().any(|v| v.len() != size) {
            return Err(AocError::InvalidInput(format!(
                "Not all rows of tile {} have the expected width of {}",
                id, size
            )));
        }

        // Pull out the edges
        fn convert<'a>(iter: impl Iterator<Item = &'a char>) -> Vec<bool> {
            iter.map(|c| *c == '#').collect()
        }
        let edges = enum_map! {
            Edge::Top => convert(lv[0].iter()),
            Edge::Bottom => convert(lv.last().unwrap().iter()),
            Edge::Left => convert(lv.iter().map(|v| &v[0])),
            Edge::Right => convert(lv.iter().map(|v| v.last().unwrap())),
        };
        let mut edges_reversed = EnumMap::default();
        for (k, v) in edges.iter() {
            let mut rv = v.clone();
            rv.reverse();
            edges_reversed[k] = rv;
        }

        Ok(Tile {
            id,
            edges,
            edges_reversed,
        })
    }
}
impl Tile {
    fn get_edge(&self, edge: Edge, rotation: Rotation) -> &[bool] {
        use Edge::*;
        use Rotation::*;

        match rotation {
            Deg0 => &self.edges[edge],
            Deg90 => match edge {
                Top => &self.edges[Right],
                Bottom => &self.edges[Left],
                Left => &self.edges_reversed[Top],
                Right => &self.edges_reversed[Bottom],
            },
            Deg180 => match edge {
                Top => &self.edges_reversed[Bottom],
                Bottom => &self.edges_reversed[Top],
                Left => &self.edges_reversed[Right],
                Right => &self.edges_reversed[Left],
            },
            Deg270 => match edge {
                Top => &self.edges_reversed[Left],
                Bottom => &self.edges_reversed[Right],
                Left => &self.edges[Bottom],
                Right => &self.edges[Top],
            },
        }
    }
}

/// Search for square root of an integer if it exists
fn sqrt(n: usize) -> Option<usize> {
    let mut i: usize = 0;
    loop {
        let s = i * i;
        if s == n {
            break Some(i);
        } else if s > n {
            break None;
        }
        i += 1;
    }
}

#[derive(Debug)]
struct TileSet {
    tiles: Vec<Tile>,
    size: usize,
}
impl FromStr for TileSet {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tiles = s
            .split("\n\n")
            .map(|s| s.parse())
            .collect::<Result<Vec<Tile>, _>>()?;

        // Verify that tiles can be placed in a square map
        let size = sqrt(tiles.len());
        if size.is_none() {
            return Err(AocError::InvalidInput(format!(
                "Tile set has {} elements, which is not a square number",
                tiles.len()
            )));
        }

        Ok(TileSet {
            tiles,
            size: size.unwrap(),
        })
    }
}

#[derive(Clone)]
struct TileSlot<'a> {
    tile: &'a Tile,
    rotation: Rotation,
}

struct TileMap<'a> {
    slots: Vec<Vec<Option<TileSlot<'a>>>>,
}
impl<'a> TileMap<'a> {
    fn new(size: usize) -> Self {
        TileMap {
            slots: vec![vec![None; size]; size],
        }
    }

    fn set(&mut self, x: usize, y: usize, tile: &'a Tile, rotation: Rotation) {
        self.slots[y][x] = Some(TileSlot { tile, rotation });
    }

    fn get(&self, x: usize, y: usize) -> &Option<TileSlot> {
        &self.slots[y][x]
    }
}

#[derive(Debug)]
struct Solver {
    tile_set: TileSet,
}
impl FromStr for Solver {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Solver {
            tile_set: s.parse()?,
        })
    }
}
impl Solver {
    fn solve(&self) -> AocResult<TileMap> {
        let map = TileMap::new(self.tile_set.size);

        fn solve_slot(x: usize, y: usize, map: TileMap, remaining: Vec<&Tile>) -> bool {
            if remaining.is_empty() {
                return true;
            }
            for (idx, tile) in remaining.into_iter().enumerate() {
                // Do we need to match to the right side of the tile to the left?
                if x > 0 {
                    let left_slot = &map.get(x - 1, y).unwrap();
                    //if tile.get_edge(Edge::LEFT, rotation)
                }

                let remaining = remaining.clone();
                remaining.remove(idx);
            }

            // TODO
            return false;
        }

        Ok(map)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 20,
    name: "Jurassic Jigsaw",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let solver: Solver = input.parse()?;
            //println!("TODO: {:?}", solver);

            // Process
            Ok(0)
        },
    ],
};
