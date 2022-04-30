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

#[derive(Clone, CharGridDebug, new)]
struct Image {
    pixels: Grid<bool>,
}
impl CharGrid<bool> for Image {
    fn get_grid(&self) -> &Grid<bool> {
        &self.pixels
    }

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
impl FromStr for Image {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Image {
            pixels: Self::grid_from_str(s)?,
        })
    }
}
impl Image {
    fn default(size: GridSize) -> Self {
        Self {
            pixels: Grid::default(size),
        }
    }

    /// Returns an image rotated 90 degrees counter-clockwise
    fn rot_90(&self) -> Self {
        let size = self.pixels.size();
        let mut out = Image::default(GridSize::new(size.y, size.x));
        for point in self.pixels.all_points() {
            out.pixels.set(
                &GridPoint::new(point.y, size.x - 1 - point.x),
                *self.pixels.get(&point),
            );
        }
        out
    }

    fn flip_hor(&self) -> Self {
        let size = self.pixels.size();
        let mut out = Image::default(*size);
        for point in self.pixels.all_points() {
            out.pixels.set(
                &GridPoint::new(size.x - 1 - point.x, point.y),
                *self.pixels.get(&point),
            )
        }
        out
    }
    fn flip_ver(&self) -> Self {
        let size = self.pixels.size();
        let mut out = Image::default(*size);
        for point in self.pixels.all_points() {
            out.pixels.set(
                &GridPoint::new(point.x, size.y - 1 - point.y),
                *self.pixels.get(&point),
            )
        }
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
        let size = self.pixels.size();
        let right_size = right.pixels.size();
        assert_eq!(
            size.y, right_size.y,
            "Images must have the same height to adjoin horizontally"
        );
        let width = size.x + right_size.x;
        let mut out = Image::default(GridSize::new(width, size.y));
        for x in 0..width {
            for y in 0..size.y {
                let point = GridPoint::new(x, y);
                out.pixels.set(
                    &point,
                    if x < size.x {
                        *self.pixels.get(&point)
                    } else {
                        *right.pixels.get(&GridPoint::new(point.x - size.x, point.y))
                    },
                )
            }
        }
        out
    }

    fn adjoin_below(&self, below: &Self) -> Self {
        let size = self.pixels.size();
        let below_size = below.pixels.size();
        assert_eq!(
            size.x, below_size.x,
            "Images must have the same width to adjoin vertically"
        );
        let height = size.y + below_size.y;
        let mut out = Image::default(GridSize::new(size.x, height));
        for x in 0..size.x {
            for y in 0..height {
                let point = GridPoint::new(x, y);
                out.pixels.set(
                    &point,
                    if y < size.y {
                        *self.pixels.get(&point)
                    } else {
                        *below.pixels.get(&GridPoint::new(point.x, point.y - size.y))
                    },
                )
            }
        }
        out
    }

    fn search(&self, image: &Self) -> Vec<GridPoint> {
        let size = self.pixels.size();
        let image_size = image.pixels.size();

        iproduct!(0..=(size.y - image_size.y), 0..=(size.x - image_size.x))
            .filter_map(|(y, x)| {
                let point = GridPoint::new(x, y);
                let sub_image = self.pixels.sub_grid(&point, *image_size);
                if image
                    .pixels
                    .all_values()
                    .zip(sub_image.all_values())
                    .all(|(pi, ps)| !pi || *ps)
                {
                    Some(point)
                } else {
                    None
                }
            })
            .collect()
    }

    fn subtract(&mut self, point: &GridPoint, image: &Self) {
        for image_point in image.pixels.all_points() {
            if *image.pixels.get(&image_point) {
                self.pixels.set(&(point + image_point), false);
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
        let size = full_image.pixels.size().x;
        if full_image.pixels.size().y != size || size < 3 {
            return Err(AocError::InvalidInput(
                format!("Tile {} must be square with at least a size of 3x3", id).into(),
            ));
        }

        // Create image of interior
        let image = Image::new(
            full_image
                .pixels
                .sub_grid(&GridPoint::new(1, 1), GridSize::new(size - 2, size - 2)),
        );

        // Pull out the edges
        let edges: EnumMap<_, Vec<bool>> = enum_map! {
            Edge::Top => full_image.pixels.row_iter(0).cloned().collect(),
            Edge::Bottom => full_image.pixels.row_iter(full_image.pixels.size().y-1).cloned().collect(),
            Edge::Left => full_image.pixels.col_iter(0).cloned().collect(),
            Edge::Right => full_image.pixels.col_iter(full_image.pixels.size().x - 1).cloned().collect(),
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
    remaining: Vec<&'a Tile>,
    slots: Grid<Option<TileSlot<'a>>>,
}
impl fmt::Debug for TileMap<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.slots.rows_iter() {
            writeln!(
                f,
                "{}",
                row.iter()
                    .map(|slot| match slot {
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
            remaining: tile_set.tiles.iter().collect(),
            slots: Grid::default(GridSize::new(size, size)),
        }
    }

    fn size(&self) -> usize {
        self.slots.size().x
    }

    fn set(&mut self, point: &GridPoint, tile: &'a Tile, transform: Transform) {
        self.slots.set(point, Some(TileSlot { tile, transform }));
    }

    fn get(&self, point: &GridPoint) -> Option<&TileSlot> {
        self.slots.get(point).as_ref()
    }

    fn stitched_image(&self) -> AocResult<Image> {
        // Check that all slots are filled
        if self.slots.all_values().any(|s| s.is_none()) {
            return Err(AocError::Process(
                "Cannot stitch image because not every slot in the map is set".into(),
            ));
        }
        Ok(self
            .slots
            .rows_iter()
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
                        let left_slot = map.get(&GridPoint::new(x - 1, y)).unwrap();
                        if tile.get_edge(Edge::Left, transform)
                            != left_slot.tile.get_edge(Edge::Right, left_slot.transform)
                        {
                            fits = false;
                        }
                    }
                    // Do we need to match the top side of the tile with the bottom
                    // side of the tile above?
                    if y > 0 {
                        let above_slot = map.get(&GridPoint::new(x, y - 1)).unwrap();
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
                        map.set(&GridPoint::new(x, y), tile, transform);
                        map.remaining.remove(tile_idx);
                        let (x, y) = if x == map.size() - 1 {
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

            let size = map.size();
            Ok(iproduct!([0, size - 1], [0, size - 1])
                .map(|(x, y)| map.get(&GridPoint::new(x, y)).unwrap().tile.id)
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

                    // Count the rough spots (i.e. points not part of a sea monster)
                    return Ok(Answer::Unsigned(
                        image.pixels.all_values().filter_count(|v| **v),
                    ));
                }
            }

            Err(AocError::Process("No sea monsters found!".into()))
        },
    ],
};
