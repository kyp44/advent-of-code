use std::str::FromStr;
use std::{cmp::Ordering, fmt};

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
use itertools::iproduct;
use itertools::Itertools;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![83775126454273],
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

#[derive(Debug, Enum)]
enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

/// Rotations and flips form a non-abelian group with eight elements.
/// These are the eight elements that are reachable from rotations
/// and flips.
#[derive(Clone, Copy, EnumIter, Display)]
enum Transform {
    Rot0,
    Rot90,
    Rot180,
    Rot270,
    FlipH,
    FlipV,
    Rot90FlipH,
    Rot90FlipV,
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
    fn get_edge(&self, edge: Edge, transform: Transform) -> &[bool] {
        use Edge::*;
        use Transform::*;

        match transform {
            Rot0 => &self.edges[edge],
            Rot90 => match edge {
                Top => &self.edges[Right],
                Bottom => &self.edges[Left],
                Left => &self.edges_reversed[Top],
                Right => &self.edges_reversed[Bottom],
            },
            Rot180 => match edge {
                Top => &self.edges_reversed[Bottom],
                Bottom => &self.edges_reversed[Top],
                Left => &self.edges_reversed[Right],
                Right => &self.edges_reversed[Left],
            },
            Rot270 => match edge {
                Top => &self.edges_reversed[Left],
                Bottom => &self.edges_reversed[Right],
                Left => &self.edges[Bottom],
                Right => &self.edges[Top],
            },
            FlipH => match edge {
                Top => &self.edges_reversed[Top],
                Bottom => &self.edges_reversed[Bottom],
                Left => &self.edges[Right],
                Right => &self.edges[Left],
            },
            FlipV => match edge {
                Top => &self.edges[Bottom],
                Bottom => &self.edges[Top],
                Left => &self.edges_reversed[Left],
                Right => &self.edges_reversed[Right],
            },
            Rot90FlipH => match edge {
                Top => &self.edges_reversed[Right],
                Bottom => &self.edges_reversed[Left],
                Left => &self.edges_reversed[Bottom],
                Right => &self.edges_reversed[Top],
            },
            Rot90FlipV => match edge {
                Top => &self.edges[Left],
                Bottom => &self.edges[Right],
                Left => &self.edges[Top],
                Right => &self.edges[Bottom],
            },
        }
    }
}

/// Search for square root of an integer if it exists
fn sqrt(n: usize) -> Option<usize> {
    let mut i: usize = 0;
    loop {
        let s = i * i;
        match s.cmp(&n) {
            Ordering::Equal => break Some(i),
            Ordering::Greater => break None,
            Ordering::Less => i += 1,
        }
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
    transform: Transform,
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
            writeln!(
                f,
                "{}",
                &(0..self.size)
                    .map(|x| match self.get(x, y) {
                        Some(t) => format!("{} {}", t.tile.id, t.transform),
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
            size,
            remaining: tile_set.tiles.iter().collect(),
            slots: vec![vec![None; size]; size],
        }
    }

    fn set(&mut self, x: usize, y: usize, tile: &'a Tile, transform: Transform) {
        self.slots[y][x] = Some(TileSlot { tile, transform });
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

        fn solve_slot(x: usize, y: usize, map: TileMap) -> Option<TileMap> {
            if map.remaining.is_empty() {
                return Some(map);
            }
            //println!("Have :\n {}", map);

            for (tile_idx, tile) in map.remaining.iter().enumerate() {
                for transform in Transform::iter() {
                    /*println!(
                        "Trying tile {} with transform {} at ({}, {})",
                        tile.id, transform, x, y
                    );*/
                    let mut fits = true;
                    // Do we need to match to the right side of the tile to the left?
                    if x > 0 {
                        let left_slot = map.get(x - 1, y).unwrap();
                        if tile.get_edge(Edge::Left, transform)
                            != left_slot.tile.get_edge(Edge::Right, left_slot.transform)
                        {
                            fits = false;
                        }
                    }
                    // Do we need to match the top side of the tile with the bottom
                    // side of the tile above?
                    if y > 0 {
                        let above_slot = map.get(x, y - 1).unwrap();
                        if tile.get_edge(Edge::Top, transform)
                            != above_slot.tile.get_edge(Edge::Bottom, above_slot.transform)
                        {
                            fits = false;
                        }
                    }

                    if fits {
                        // The tile fits, so place it and work on the next tile
                        //println!("It fit!");
                        let mut map = map.clone();
                        map.set(x, y, tile, transform);
                        map.remaining.remove(tile_idx);
                        let (x, y) = if x == map.size - 1 {
                            (0, y + 1)
                        } else {
                            (x + 1, y)
                        };
                        if let Some(map) = solve_slot(x, y, map) {
                            return Some(map);
                        }
                    }
                }
            }

            // Could not complete the map
            None
        }

        match solve_slot(0, 0, map) {
            Some(map) => {
                let size = map.size;
                //println!("Solution\n: {}", map);
                Ok(iproduct!([0, size - 1], [0, size - 1])
                    .map(|(x, y)| map.get(x, y).unwrap().tile.id)
                    .product())
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
