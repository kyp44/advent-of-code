use crate::aoc::prelude::*;
use enum_map::{enum_map, Enum, EnumMap};
use itertools::iproduct;
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::line_ending,
    sequence::{delimited, pair},
    Finish,
};
use std::str::FromStr;
use std::{cmp::Ordering, fmt};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(83775126454273), Unsigned(1993)],
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
    vec![20899048083289, 273u64].answer_vec()
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

#[derive(Clone, CharGridDebug)]
struct Image {
    size: GridSize,
    pixels: Box<[Box<[bool]>]>,
}
impl Grid<bool> for Image {
    fn default(size: GridSize) -> Self {
        Self {
            size,
            pixels: vec![vec![false; size.x].into_boxed_slice(); size.y].into_boxed_slice(),
        }
    }

    fn size(&self) -> &GridSize {
        &self.size
    }

    fn get(&self, point: &GridPoint) -> &bool {
        &self.pixels[point.y][point.x]
    }

    fn set(&mut self, point: &GridPoint, value: bool) {
        self.pixels[point.y][point.x] = value;
    }
}
impl CharGrid<bool> for Image {
    fn from_char(c: char) -> Option<bool> {
        match c {
            '#' => Some(true),
            '.' => Some(false),
            _ => None,
        }
    }

    fn to_char(e: &bool) -> char {
        if *e {
            '#'
        } else {
            '.'
        }
    }
}

impl Image {
    fn set_todo(&mut self, rows: impl Iterator<Item = impl Iterator<Item = bool>>) {
        todo!()
        /*for (y, row) in (0..self.size.y).zip(rows) {
            for (x, v) in (0..self.size.0).zip(row) {
                self.pixels[y][x] = v;
            }
        }*/
    }

    fn rot_90(&self) -> Self {
        todo!()
        /*
        let mut out = Image::default(self.size);
        out.set_todo(
            (0..self.size.0)
                .rev()
                .map(|x| (0..self.size.y).map(move |y| self.pixels[y][x])),
        );
        out*/
    }

    fn flip_hor(&self) -> Self {
        let mut out = Image::default(self.size);
        out.set_todo(self.pixels.iter().map(|row| row.iter().rev().copied()));
        out
    }
    fn flip_ver(&self) -> Self {
        let mut out = Image::default(self.size);
        out.set_todo(self.pixels.iter().rev().map(|row| row.iter().copied()));
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
        todo!()
        /*
        assert_eq!(
            self.size.y, right.size.y,
            "Images must have the same height to adjoin horizontally"
        );
        let mut out = Image::default((self.size.x + right.size.x, self.size.y));
        out.set_todo(
            self.pixels
                .iter()
                .zip(right.pixels.iter())
                .map(|(l, r)| l.iter().chain(r.iter()).copied()),
        );
        out*/
    }

    fn adjoin_below(&self, below: &Self) -> Self {
        todo!()
        /*
        assert_eq!(
            self.size.x, below.size.x,
            "Images must have the same width to adjoin vertically"
        );
        let mut out = Image::default((self.size.x, self.size.y + below.size.y));
        out.set_todo(
            self.pixels
                .iter()
                .chain(below.pixels.iter())
                .map(|row| row.iter().copied()),
        );
        out*/
    }

    fn slice(&self, point: &(usize, usize), width: usize, height: usize) -> Box<[&[bool]]> {
        self.pixels[point.1..point.1 + height]
            .iter()
            .map(|row| &row[point.0..point.0 + width])
            .collect::<Vec<&[bool]>>()
            .into_boxed_slice()
    }

    fn slice_mut(
        &mut self,
        point: &(usize, usize),
        width: usize,
        height: usize,
    ) -> Box<[&mut [bool]]> {
        self.pixels[point.1..point.1 + height]
            .iter_mut()
            .map(|row| &mut row[point.0..point.0 + width])
            .collect::<Vec<&mut [bool]>>()
            .into_boxed_slice()
    }

    fn search(&self, image: &Self) -> Vec<(usize, usize)> {
        iproduct!(
            0..=(self.size.y - image.size.y),
            0..=(self.size.x - image.size.x)
        )
        .filter(|(y, x)| {
            self.slice(&(*x, *y), image.size.x, image.size.y)
                .iter()
                .flat_map(|row| row.iter())
                .zip(image.pixels.iter().flat_map(|row| row.iter()))
                .all(|(p, sp)| !sp || *p)
        })
        .map(|(y, x)| (x, y))
        .collect()
    }

    fn subtract(&mut self, point: &(usize, usize), image: &Self) {
        let mut slice = self.slice_mut(point, image.size.x, image.size.y);
        for (p, sp) in slice
            .iter_mut()
            .flat_map(|row| row.iter_mut())
            .zip(image.pixels.iter().flat_map(|row| row.iter()))
        {
            if *sp {
                *p = false;
            }
        }
    }
}

#[derive(Debug)]
struct Tile {
    id: u64,
    image: Image,
    edges: EnumMap<Edge, Vec<bool>>,
    edges_reversed: EnumMap<Edge, Vec<bool>>,
}
impl FromStr for Tile {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (image_str, id) = delimited::<_, _, _, _, NomParseError, _, _, _>(
            tag("Tile "),
            nom::character::complete::u64,
            pair(tag(":"), line_ending),
        )(s)
        .finish()?;
        let full_image = Image::from_str(image_str)?;

        // Verify the tile dimensions
        let size = full_image.size.x;
        if size != full_image.size.y || size < 3 {
            return Err(AocError::InvalidInput(
                format!("Tile {} must be square with at least a size of 3x3", id).into(),
            ));
        }

        todo!()
        /*// Create image of interior
        let mut image = Image::default((size - 2, size - 2));
        image.set_????(
            full_image.pixels[1..size - 1]
                .iter()
                .map(|row| row[1..size - 1].iter().copied()),
        );

        // Pull out the edges
        let edges: EnumMap<_, Vec<bool>> = enum_map! {
            Edge::Top => full_image.pixels[0].to_vec(),
            Edge::Bottom => full_image.pixels.last().unwrap().to_vec(),
            Edge::Left => full_image.pixels.iter().map(|row| row[0]).collect(),
            Edge::Right => full_image.pixels.iter().map(|row| *row.last().unwrap()).collect(),
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
        })*/
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
            return Err(AocError::InvalidInput(
                format!(
                    "Tile set has {} elements, which is not a square number",
                    tiles.len()
                )
                .into(),
            ));
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
impl fmt::Debug for TileMap<'_> {
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

    fn stitched_image(&self) -> AocResult<Image> {
        // Check that all slots are filled
        if self
            .slots
            .iter()
            .flat_map(|row| row.iter())
            .any(|slot| slot.is_none())
        {
            return Err(AocError::Process(
                "Cannot stitch image because not every slot in the map is set".into(),
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
            //println!("Have :\n {:?}", map);

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

        solve_slot(0, 0, map).ok_or_else(|| AocError::Process("Could not find a solution".into()))
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
            //println!("Solution\n: {:?}", map);

            let size = map.size;
            Ok(iproduct!([0, size - 1], [0, size - 1])
                .map(|(x, y)| map.get(x, y).unwrap().tile.id)
                .product::<u64>()
                .into())
        },
        // Part b)
        |input| {
            // Generation
            let solver: Solver = input.parse()?;

            // Process
            let image = solver.solve()?.stitched_image()?;
            let sea_monster: Image = Image::from_str(
                "                  # 
#    ##    ##    ###
 #  #  #  #  #  #   ",
            )?;

            for transform in Transform::iter() {
                let mut image = image.transformed(transform);
                let found_coords = image.search(&sea_monster);
                if !found_coords.is_empty() {
                    // Subtract out the sea monster points
                    for point in found_coords {
                        image.subtract(&point, &sea_monster)
                    }

                    // Count the rough spots
                    return Ok(Answer::Unsigned(
                        image
                            .pixels
                            .iter()
                            .flat_map(|row| row.iter().copied())
                            .filter_count(|p| *p),
                    ));
                }
            }

            Err(AocError::Process("No sea monsters found!".into()))
        },
    ],
};
