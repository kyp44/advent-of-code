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
    vec![Some(20899048083289), Some(273)]
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

#[derive(Clone)]
struct Image<T> {
    width: usize,
    height: usize,
    pixels: Box<[Box<[T]>]>,
}
impl<T> Image<T>
where
    T: Default + Copy,
{
    fn new(width: usize, height: usize) -> Self {
        Image {
            width,
            height,
            pixels: vec![vec![T::default(); width].into_boxed_slice(); height].into_boxed_slice(),
        }
    }

    fn set(&mut self, rows: impl Iterator<Item = impl Iterator<Item = T>>) {
        for (y, row) in (0..self.height).zip(rows) {
            for (x, v) in (0..self.width).zip(row) {
                self.pixels[y][x] = v;
            }
        }
    }

    fn rot_90(&self) -> Self {
        let mut out = Image::new(self.height, self.width);
        out.set(
            (0..self.width)
                .rev()
                .map(|x| (0..self.height).map(move |y| self.pixels[y][x])),
        );
        out
    }

    fn flip_hor(&self) -> Self {
        let mut out = Image::new(self.width, self.height);
        out.set(self.pixels.iter().map(|row| row.iter().rev().map(|p| *p)));
        out
    }
    fn flip_ver(&self) -> Self {
        let mut out = Image::new(self.width, self.height);
        out.set(
            self.pixels
                .iter()
                .rev()
                .map(|row| row.into_iter().map(|p| *p)),
        );
        out
    }

    fn transformed(&self, transform: Transform) -> Self {
        use Transform::*;
        match transform {
            Rot0 => self.clone(),
            Rot90 => self.rot_90(),
            Rot180 => self.rot_90().rot_90(),
            Rot270 => self.rot_90().rot_90().rot_90(),
            FlipH => self.flip_hor(),
            FlipV => self.flip_ver(),
            Rot90FlipH => self.rot_90().flip_hor(),
            Rot90FlipV => self.rot_90().flip_ver(),
        }
    }

    fn adjoin_right(&self, right: &Self) -> Self {
        assert_eq!(
            self.height, right.height,
            "Images must have the same height to adjoin horizontally"
        );
        let mut out = Image::new(self.width + right.width, self.height);
        out.set(
            self.pixels
                .iter()
                .zip(right.pixels.iter())
                .map(|(l, r)| l.iter().chain(r.iter()).map(|p| *p)),
        );
        out
    }

    fn adjoin_below(&self, below: &Self) -> Self {
        assert_eq!(
            self.width, below.width,
            "Images must have the same width to adjoin vertically"
        );
        let mut out = Image::new(self.width, self.height + below.height);
        out.set(
            self.pixels
                .iter()
                .chain(below.pixels.iter())
                .map(|row| row.iter().map(|p| *p)),
        );
        out
    }
}
impl fmt::Display for Image<bool> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}",
            self.pixels
                .iter()
                .map(|row| row
                    .iter()
                    .map(|p| if *p { '#' } else { '.' })
                    .collect::<String>())
                .join("\n")
        )
    }
}
impl fmt::Debug for Image<bool> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Debug)]
struct Tile {
    id: u64,
    image: Image<bool>,
    edges: EnumMap<Edge, Vec<bool>>,
    edges_reversed: EnumMap<Edge, Vec<bool>>,
}
impl FromStr for Tile {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (id, rows) = map::<_, _, _, ParseError, _, _>(
            pair(
                delimited(tag("Tile "), digit1, pair(tag(":"), line_ending)),
                separated_list1(line_ending, many1(one_of(".#"))),
            ),
            |(ids, lv): (&str, Vec<Vec<char>>)| (ids.parse().unwrap(), lv),
        )(s)
        .finish()
        .discard_input()?;

        // Verify the tile dimensions
        let size = rows[0].len();
        if size < 3 {
            return Err(AocError::InvalidInput(format!(
                "Tile {} must be at least 3x3",
                id
            )));
        }
        if rows.len() != size {
            return Err(AocError::InvalidInput(format!(
                "Tile {} did not have the expected height of {}",
                id, size
            )));
        }
        if rows.iter().any(|v| v.len() != size) {
            return Err(AocError::InvalidInput(format!(
                "Not all rows of tile {} have the expected width of {}",
                id, size
            )));
        }

        // Converts iterator of chars to boolean pixels
        fn convert<'a>(
            iter: impl Iterator<Item = &'a char> + 'a,
        ) -> impl Iterator<Item = bool> + 'a {
            iter.map(|c| *c == '#')
        }

        // Create image of interior
        let mut image = Image::new(size - 2, size - 2);
        image.set(
            rows[1..size - 1]
                .iter()
                .map(|row| convert(row[1..size - 1].iter())),
        );

        // Pull out the edges
        let edges: EnumMap<_, Vec<bool>> = enum_map! {
            Edge::Top => convert(rows[0].iter()).collect(),
            Edge::Bottom => convert(rows.last().unwrap().iter()).collect(),
            Edge::Left => convert(rows.iter().map(|v| &v[0])).collect(),
            Edge::Right => convert(rows.iter().map(|v| v.last().unwrap())).collect(),
        };
        let mut edges_reversed = EnumMap::default();
        for (k, v) in edges.iter() {
            let mut rv = v.clone();
            rv.reverse();
            edges_reversed[k] = rv;
        }

        Ok(Tile {
            id,
            image,
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

    fn stitched_image(&self) -> AocResult<Image<bool>> {
        // Check that all slots are filled
        if self
            .slots
            .iter()
            .map(|row| row.iter())
            .flatten()
            .any(|slot| slot.is_none())
        {
            return Err(AocError::Process(
                "Cannot stitch image because not every slot in the map is set".to_string(),
            ));
        }
        Ok(self
            .slots
            .iter()
            .map(|row| {
                row.iter()
                    .map(|slot| {
                        let slot = slot.as_ref().unwrap();
                        slot.tile.image.transformed(slot.transform)
                    })
                    .reduce(|left, right| left.adjoin_right(&right))
                    .unwrap()
            })
            .reduce(|upper, lower| upper.adjoin_below(&lower))
            .unwrap())
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

        solve_slot(0, 0, map)
            .ok_or_else(|| AocError::Process("Could not find a solution".to_string()))
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
            let map = solver.solve()?;
            //println!("Solution\n: {}", map);

            let size = map.size;
            Ok(iproduct!([0, size - 1], [0, size - 1])
                .map(|(x, y)| map.get(x, y).unwrap().tile.id)
                .product())
        },
        // Part b)
        |input| {
            // Generation
            let solver: Solver = input.parse()?;

            // Process
            let map = solver.solve()?;
            println!("{}", map);
            println!("{}", map.stitched_image()?.flip_hor().rot_90());

            Ok(0)
        },
    ],
};
