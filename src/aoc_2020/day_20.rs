use aoc::prelude::*;
use std::str::FromStr;
use strum::IntoEnumIterator;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "Tile 2311:
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
..#.###...";
            answers = unsigned![20899048083289, 273];
        }
        actual_answers = unsigned![83775126454273, 1993];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::tree_search::new::{BasicSolutionState, GlobalStateTreeNode, NodeAction};
    use cgmath::EuclideanSpace;
    use derive_more::{Deref, From, Into};
    use derive_new::new;
    use enum_map::{enum_map, Enum, EnumMap};
    use itertools::{iproduct, Itertools};
    use nom::{
        bytes::complete::tag,
        character::complete::line_ending,
        sequence::{delimited, pair},
        Finish,
    };
    use std::rc::Rc;
    use std::{cmp::Ordering, fmt};
    use strum_macros::{Display, EnumIter};

    /// An edge of an image.
    #[derive(Debug, Enum)]
    enum Edge {
        /// The top edge.
        Top,
        /// The bottom edge.
        Bottom,
        /// The left edge.
        Left,
        /// The right edge.
        Right,
    }

    /// A transformation that can be applied to an image.
    ///
    /// NOTE: Rotations and flips form a non-abelian group with eight elements.
    /// These are the eight transformations that are reachable from rotations
    /// and flips.
    #[derive(Clone, Copy, EnumIter, Display)]
    pub enum Transform {
        /// Rotate 0 degrees, that is the identity transformation.
        Rot0,
        /// Rotate 90 degrees counterclockwise.
        Rot90,
        /// Rotate 180 degrees.
        Rot180,
        /// Rotate 270 degrees counterclockwise.
        Rot270,
        /// Flip horizontally.
        FlipH,
        /// Flip vertically.
        FlipV,
        /// Rotate 90 degrees counterclockwise, then flip horizontally.
        Rot90FlipH,
        /// Rotate 90 degrees counterclockwise, then flip vertically.
        Rot90FlipV,
    }

    /// A boolean pixel in the image.
    #[derive(Deref, From, Into, Default, Clone, Copy)]
    pub struct Pixel(bool);
    impl TryFrom<char> for Pixel {
        type Error = ();

        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                '#' => Ok(true.into()),
                '.' => Ok(false.into()),
                ' ' => Ok(false.into()),
                _ => Err(()),
            }
        }
    }
    impl fmt::Debug for Pixel {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", if **self { '#' } else { '.' })
        }
    }

    /// A general monochrome image, which can be parsed from text input.
    #[derive(Clone, new, Debug)]
    pub struct Image {
        /// Grid of pixels.
        pixels: Grid<Pixel>,
    }
    impl Image {
        /// Returns this image rotated 90 degrees counter-clockwise.
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

        /// Returns this image flipped horizontally.
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

        /// Returns this image flipped vertically.
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

        /// Returns this image with some transformation applied.
        pub fn transformed(&self, transform: Transform) -> Self {
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

        /// Returns this image with another adjoined on the right.
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

        /// Returns this image with another adjoined below.
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

        /// Searches the image for a sub-image mask.
        ///
        /// Set pixels of the sub-image  must be set pixels in this image, and
        /// unset pixels in the sub-image can be any pixels in this image.
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
                        .all(|(pi, ps)| !**pi || **ps)
                    {
                        Some(point)
                    } else {
                        None
                    }
                })
                .collect()
        }

        /// Subtracts a smaller sub-image from this image at the specified
        /// upper left coordinates.
        ///
        /// Pixels in the sub image that are set are unset in this image,
        /// and those that are unset in the sub-image are left unchanged
        /// in this image.
        fn subtract(&mut self, point: &GridPoint, image: &Self) {
            for image_point in image.pixels.all_points() {
                if **image.pixels.get(&image_point) {
                    self.pixels
                        .set(&(point + image_point.to_vec()), false.into());
                }
            }
        }

        /// Fins the sea monster in whatever orientation necessary and subtracts it,
        /// returning the subtracted image.
        pub fn find_and_subtract_sea_monster(&self) -> AocResult<Self> {
            let sea_monster: Image = Image::from_grid_str(
                "                  # 
#    ##    ##    ###
 #  #  #  #  #  #   ",
            )?;

            for transform in Transform::iter() {
                let mut image = self.transformed(transform);
                let found_coords = image.search(&sea_monster);
                if !found_coords.is_empty() {
                    // Subtract out the sea monster points
                    for point in found_coords {
                        image.subtract(&point, &sea_monster)
                    }

                    // Count the rough spots (i.e. points not part of a sea monster)
                    return Ok(image);
                }
            }

            Err(AocError::Process("No sea monsters found!".into()))
        }

        /// Counts the set pixels.
        pub fn count_set_pixels(&self) -> u64 {
            self.pixels.all_values().filter_count(|v| ***v)
        }
    }
    impl From<Grid<Pixel>> for Image {
        fn from(value: Grid<Pixel>) -> Self {
            Self::new(value)
        }
    }

    /// A tile (an image from the satellite camera array), which can be parsed from text input.
    #[derive(Debug)]
    struct Tile {
        /// Tile ID.
        id: u64,
        /// Image of the tile.
        image: Image,
        /// Map of edge to the image edge on that side.
        edges: EnumMap<Edge, Vec<bool>>,
        /// The `edges` map with a reversed edge vector.
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
            let full_image = Image::from_grid_str(image_str)?;

            // Verify the tile dimensions
            let size = full_image.pixels.size().x;
            if full_image.pixels.size().y != size || size < 3 {
                return Err(AocError::InvalidInput(
                    format!("Tile {id} must be square with at least a size of 3x3").into(),
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
                Edge::Top => full_image.pixels.row_iter(0).map(|sb| **sb).collect(),
                Edge::Bottom => full_image.pixels.row_iter(full_image.pixels.size().y-1).map(|sb| **sb).collect(),
                Edge::Left => full_image.pixels.column_iter(0).map(|sb| **sb).collect(),
                Edge::Right => full_image.pixels.column_iter(full_image.pixels.size().x - 1).map(|sb| **sb).collect(),
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
        /// Gets an edge of this image if a transform were to be applied to the image.
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

    /// Searches for square root of an integer if it exists.
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

    /// Set of tiles, which can be parsed from text input.
    #[derive(Debug)]
    struct TileSet {
        /// List of tiles.
        tiles: Vec<Tile>,
        /// The width and height in number of tiles if the tiles are arranged in a square.
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

    /// A slot for a tile in a square image formed by tiles.
    #[derive(Clone)]
    struct TileSlot {
        /// The tile in this slot.
        tile: Rc<Tile>,
        /// How the tile must be transformed to fit in the overall image.
        transform: Transform,
    }

    /// A square map of tiles formed into a larger image.
    #[derive(Clone)]
    pub struct TileMap {
        /// Remaining tiles that need to be placed.
        remaining: Vec<Rc<Tile>>,
        /// The square grid of tile slots, which may be empty.
        slots: Grid<Option<TileSlot>>,
        /// Current tile that needs to be placed when solving.
        placement_tile: GridPoint,
    }
    impl fmt::Debug for TileMap {
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
    impl TileMap {
        /// Creates a new map from set of tiles, initially with all tiles
        /// needing to be placed.
        fn new(tile_set: TileSet) -> Self {
            let size = tile_set.size;
            TileMap {
                remaining: tile_set.tiles.into_iter().map(Rc::new).collect(),
                slots: Grid::default(GridSize::new(size, size)),
                placement_tile: GridPoint::origin(),
            }
        }

        /// Returns the width and height of the square map in tiles.
        fn size(&self) -> usize {
            self.slots.size().x
        }

        /// Sets a tile in the map.
        fn set(&mut self, point: &GridPoint, tile: Rc<Tile>, transform: Transform) {
            self.slots.set(point, Some(TileSlot { tile, transform }));
        }

        /// Gets the tile at a point in the map, if one is placed there.
        fn get(&self, point: &GridPoint) -> Option<&TileSlot> {
            self.slots.get(point).as_ref()
        }

        /// Verifies that the map is filled.
        fn verify_filled(&self) -> AocResult<()> {
            if self.slots.all_values().any(|s| s.is_none()) {
                Err(AocError::Process(
                    "The tile map is not completely filled".into(),
                ))
            } else {
                Ok(())
            }
        }

        /// Results a stitched image of the tile map if possible, that is if all
        /// the tiles have been placed.
        pub fn stitched_image(&self) -> AocResult<Image> {
            self.verify_filled()?;

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

        /// Verifies that the map has been filled and, if so, returns
        /// the product of the four corner tile IDs.
        pub fn corner_id_product(&self) -> AocResult<u64> {
            self.verify_filled()?;

            let size = self.size();
            Ok(iproduct!([0, size - 1], [0, size - 1])
                .map(|(x, y)| self.get(&GridPoint::new(x, y)).unwrap().tile.id)
                .product::<u64>())
        }
    }

    impl GlobalStateTreeNode for TileMap {
        type GlobalState = BasicSolutionState<Self>;

        fn recurse_action(self, global_state: &mut Self::GlobalState) -> NodeAction<Self> {
            if self.remaining.is_empty() {
                global_state.set_solution(self.clone());
                return NodeAction::Complete;
            }
            //println!("Have :\n {:?}", map);

            let (x, y) = (self.placement_tile.x, self.placement_tile.y);

            let children: Vec<Self> = self
                .remaining
                .iter()
                .enumerate()
                .cartesian_product(Transform::iter())
                .filter_map(|((tile_idx, tile), transform)| {
                    /*println!(
                        "Trying tile {} with transform {} at ({}, {})",
                        tile.id, transform, x, y
                    );*/
                    let mut fits = true;
                    // Do we need to match to the right side of the tile to the left?
                    if x > 0 {
                        let left_slot = self.get(&GridPoint::new(x - 1, y)).unwrap();
                        if tile.get_edge(Edge::Left, transform)
                            != left_slot.tile.get_edge(Edge::Right, left_slot.transform)
                        {
                            fits = false;
                        }
                    }
                    // Do we need to match the top side of the tile with the bottom
                    // side of the tile above?
                    if y > 0 {
                        let above_slot = self.get(&GridPoint::new(x, y - 1)).unwrap();
                        if tile.get_edge(Edge::Top, transform)
                            != above_slot.tile.get_edge(Edge::Bottom, above_slot.transform)
                        {
                            fits = false;
                        }
                    }

                    if fits {
                        // The tile fits, so place it and work on the next tile
                        //println!("It fit!");
                        let mut map = self.clone();
                        map.set(&GridPoint::new(x, y), tile.clone(), transform);
                        map.remaining.remove(tile_idx);
                        let (x, y) = if x == map.size() - 1 {
                            (0, y + 1)
                        } else {
                            (x + 1, y)
                        };
                        map.placement_tile = GridPoint::new(x, y);
                        Some(map)
                    } else {
                        None
                    }
                })
                .collect();

            if children.is_empty() {
                NodeAction::Stop
            } else {
                NodeAction::Continue(children)
            }
        }
    }

    /// Solver for the problem, which can be parsed from text input.
    #[derive(Debug)]
    pub struct Solver {
        /// Set of tiles.
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
        /// Solves the set by finding a filled tile map such that all the tile edges match
        /// up correctly.
        ///
        /// Note that this is correct only up to rotations of the entire map.
        pub fn solve(self) -> AocResult<TileMap> {
            TileMap::new(self.tile_set)
                .traverse_tree(BasicSolutionState::default())
                .solution()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 20,
    name: "Jurassic Jigsaw",
    preprocessor: Some(|input| Ok(Box::new(input.parse::<Solver>()?.solve()?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<TileMap>()?.corner_id_product()?.into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<TileMap>()?
                .stitched_image()?
                .find_and_subtract_sea_monster()?
                .count_set_pixels()
                .into())
        },
    ],
};
