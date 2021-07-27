use std::fmt;
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
use itertools::Itertools;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

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

#[derive(Clone, Copy, EnumIter, Display)]
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

#[derive(Clone)]
struct TileMap<'a> {
    size: usize,
    remaining: Vec<&'a Tile>,
    slots: Vec<Vec<Option<TileSlot<'a>>>>,
}
impl fmt::Display for TileMap<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.size {
            write!(
                f,
                "{}\n",
                &(0..self.size)
                    .map(|x| match self.get(x, y) {
                        Some(t) => format!("{} {}", t.tile.id, t.rotation),
                        None => "-".to_string(),
                    })
                    .join(" | "),
            )?;
        }
        Ok(())
    }
}
impl<'a> TileMap<'a> {
    fn new(tile_set: &'a TileSet) -> Self {
        let size = tile_set.size;
        TileMap {
            size: size,
            remaining: tile_set.tiles.iter().collect(),
            slots: vec![vec![None; size]; size],
        }
    }

    fn set(&mut self, x: usize, y: usize, tile: &'a Tile, rotation: Rotation) {
        self.slots[y][x] = Some(TileSlot { tile, rotation });
    }

    fn get(&self, x: usize, y: usize) -> Option<&TileSlot> {
        self.slots[y][x].as_ref()
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
    fn solve(&self) -> AocResult<u64> {
        let map = TileMap::new(&self.tile_set);

        fn solve_slot<'a>(x: usize, y: usize, map: TileMap<'a>) -> Option<TileMap<'a>> {
            if map.remaining.is_empty() {
                return Some(map);
            }
            for (tile_idx, tile) in map.remaining.iter().enumerate() {
                for rotation in Rotation::iter() {
                    // Do we need to match to the right side of the tile to the left?
                    if x > 0 {
                        let left_slot = map.get(x - 1, y).unwrap();
                        if tile.get_edge(Edge::Left, rotation)
                            != left_slot.tile.get_edge(Edge::Right, left_slot.rotation)
                        {
                            break;
                        }
                    }
                    // Do we need to match the top side of the tile with the bottom
                    // side of the tile above?
                    if y > 0 {
                        let above_slot = map.get(x, y - 1).unwrap();
                        if tile.get_edge(Edge::Top, rotation)
                            != above_slot.tile.get_edge(Edge::Bottom, above_slot.rotation)
                        {
                            break;
                        }
                    }

                    // The tile fits, so place it and work on the next tile
                    let mut map = map.clone();
                    map.set(x, y, tile, rotation);
                    map.remaining.remove(tile_idx);
                    println!("Trying:\n {}", map);
                    let (x, y) = if x == map.size {
                        (0, y + 1)
                    } else {
                        (x + 1, y)
                    };
                    if let Some(map) = solve_slot(x, y, map) {
                        return Some(map);
                    }
                }
            }

            // Could not complete the map
            None
        }

        match solve_slot(0, 0, map) {
            Some(map) => {
                //let size = self.tile_set.size;
                println!("Solution\n: {}", map);
                Ok(0)
            }
            None => Err(AocError::Process("Could not find a solution".to_string())),
        }
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

            // Process
            solver.solve()
        },
    ],
};
