use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "        ...#
        .#..
        #...
        ....
...#.......#
........#...
..#....#....
..........#.
        ...#....
        .....#..
        .#......
        ......#.

10R5L5R10L4R5L5";
            answers = unsigned![6032, 5031];
        }
        actual_answers = unsigned![103224, 189097];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::{collections::HashSet, iter::FusedIterator, ops::Neg, slice::Iter, str::FromStr};

    use aoc::{
        grid::StdBool,
        tree_search::{GlobalStateTreeNode, NodeAction},
    };
    use derive_new::new;
    use euclid::{Point2D, Size2D, Vector2D};
    use itertools::Itertools;
    use nom::{branch::alt, bytes::complete::tag, combinator::map, multi::many1};
    use strum::{Display, EnumIter, IntoEnumIterator};

    use super::*;

    /// Types of spaces in the main grid.
    #[derive(Clone, Copy, Default, PartialEq, Eq)]
    pub enum Space {
        /// Nothing, not a valid space.
        #[default]
        Void,
        /// An open space that can be moved through.
        Open,
        /// A wall that cannot be moved through.
        Wall,
    }
    impl Space {
        /// Returns whether the space is empty, that is a [`Void`](Self::Void)
        /// space.
        pub fn is_empty(&self) -> bool {
            self == &Self::Void
        }
    }
    impl TryFrom<char> for Space {
        type Error = ();

        fn try_from(value: char) -> Result<Self, Self::Error> {
            Ok(match value {
                ' ' | '_' => Self::Void,
                '.' => Self::Open,
                '#' => Self::Wall,
                _ => return Err(()),
            })
        }
    }
    impl std::fmt::Debug for Space {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Self::Void => ' ',
                    Self::Open => '.',
                    Self::Wall => '#',
                }
            )
        }
    }

    /// A single step in the movement instruction list, which can be
    /// parsed from text input.
    #[derive(Debug)]
    enum Step {
        /// Attempt to walk forward a certain number of spaces.
        WalkForward(u8),
        /// Turn left, remaining in the same space.
        TurnLeft,
        /// Turn right, remaining in the same space.
        TurnRight,
    }
    impl Parsable for Step {
        fn parser(input: &'_ str) -> NomParseResult<&str, Self> {
            alt((
                map(nom::character::complete::u8, Self::WalkForward),
                map(tag("L"), |_| Self::TurnLeft),
                map(tag("R"), |_| Self::TurnRight),
            ))
            .parse(input)
        }
    }

    /// A list of steps, which can be parsed from text input.
    struct ParsePath(Vec<Step>);
    impl Parsable for ParsePath {
        fn parser(input: &'_ str) -> NomParseResult<&str, Self> {
            map(many1(Step::parser), ParsePath).parse(input)
        }
    }

    /// A direction on a grid.
    #[derive(Debug, Clone, Copy, EnumIter)]
    pub enum Direction {
        /// Up with decreasing `y`.
        Up,
        /// Down with increasing `y`.
        Down,
        /// Left with decreasing `x`.
        Left,
        /// Right with increasing `x`.
        Right,
    }
    impl Direction {
        /// Returns the displacement vector to move a single space in this
        /// direction.
        pub fn as_vector<U>(&self) -> Vector2D<isize, U> {
            match self {
                Direction::Up => -Vector2D::<isize, U>::unit_y(),
                Direction::Down => Vector2D::unit_y(),
                Direction::Left => -Vector2D::<isize, U>::unit_x(),
                Direction::Right => Vector2D::unit_x(),
            }
        }

        /// Returns the direction you would face if turning left from this
        /// direction.
        pub fn turn_left(&self) -> Self {
            match self {
                Direction::Up => Self::Left,
                Direction::Down => Self::Right,
                Direction::Left => Self::Down,
                Direction::Right => Self::Up,
            }
        }

        /// Returns the direction you would face if turning right from this
        /// direction.
        pub fn turn_right(&self) -> Self {
            -self.turn_left()
        }

        /// Returns the value assigned to facing this direction for the password
        /// calculation.
        pub fn facing_value(&self) -> u64 {
            match self {
                Direction::Up => 3,
                Direction::Down => 1,
                Direction::Left => 2,
                Direction::Right => 0,
            }
        }
    }
    impl Neg for Direction {
        type Output = Self;

        fn neg(self) -> Self::Output {
            match self {
                Direction::Up => Self::Down,
                Direction::Down => Self::Up,
                Direction::Left => Self::Right,
                Direction::Right => Self::Left,
            }
        }
    }

    /// The map the monkeys give you, which represents how to determine the
    /// password and can be parsed from text input.
    pub struct MonkeyMap {
        /// The main grid for the map.
        grid: Grid<Space, MainGrid>,
        /// The list of steps taken to move around on the main grid.
        path: Vec<Step>,
    }
    impl FromStr for MonkeyMap {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let secs = s.sections(2)?;

            Ok(Self {
                grid: Grid::from_str_default(secs[0])?,
                path: ParsePath::from_str(secs[1])?.0,
            })
        }
    }
    impl MonkeyMap {
        /// Returns an iterator that traces key points on the correct path for a
        /// given part `P` of the problem.
        fn path_traversal<'a, P: Part<'a>>(&'a self) -> AocResult<PathTraversal<'a, P>> {
            let part = P::new(&self.grid)?;
            let position = part.next_space(&Position::default())?;

            if *self.grid.get(&position.point) != Space::Open {
                return Err(AocError::Process("The initial position is not open".into()));
            }

            Ok(PathTraversal {
                part,
                monkey_map: self,
                path: self.path.iter(),
                next_position: Some(position),
            })
        }

        /// Determines the correct password for this map and part `P` of the
        /// problem.
        pub fn password<'a, P: Part<'a>>(&'a self) -> AocResult<u64> {
            self.path_traversal::<P>()?
                .last()
                .ok_or(AocError::NoSolution)?
                .map(|p| p.password())
        }
    }

    /// Behavior different for each part of the problem.
    pub trait Part<'a>: Sized {
        /// Creates a new part object given the main grid.
        fn new(grid: &'a Grid<Space, MainGrid>) -> AocResult<Self>;

        /// Returns the new position if we were to try to walk forward a single
        /// space, moving ahead to the next tile for the part if
        /// necessary.
        ///
        /// This does not check the content of next space, only its [`Position`]
        /// is returned. However, the space is guaranteed not to be a
        /// [`Space::Void`].
        fn next_space(&self, position: &Position) -> AocResult<Position>;
    }

    /// Behavior for part one.
    pub struct PartOne<'a> {
        /// The main grid.
        grid: &'a Grid<Space, MainGrid>,
    }
    impl<'a> Part<'a> for PartOne<'a> {
        fn new(grid: &'a Grid<Space, MainGrid>) -> AocResult<Self> {
            Ok(Self { grid })
        }

        fn next_space(&self, position: &Position) -> AocResult<Position> {
            let mut point = position.point.try_cast().unwrap();
            let size = self.grid.size().try_cast().unwrap();
            let dir_vec = position.facing.as_vector();

            Ok(loop {
                point = (point + dir_vec).rem_euclid(&size);

                if !self.grid.get_any(&point).unwrap().is_empty() {
                    break Position::new(point.try_cast().unwrap(), position.facing);
                }
            })
        }
    }

    /// The six surfaces of a cube, which is primarily just  a way to refer to
    /// them.
    #[derive(Debug, Clone, Copy, Display, PartialEq, Eq)]
    enum CubeSurface {
        /// The front surface.
        Front,
        /// The top surface.
        Top,
        /// The right surface.
        Right,
        /// The left surface.
        Left,
        /// The bottom surface.
        Bottom,
        /// The back surface.
        Back,
    }
    impl Neg for CubeSurface {
        type Output = Self;

        fn neg(self) -> Self::Output {
            match self {
                CubeSurface::Front => Self::Back,
                CubeSurface::Top => Self::Bottom,
                CubeSurface::Right => Self::Left,
                CubeSurface::Left => Self::Right,
                CubeSurface::Bottom => Self::Top,
                CubeSurface::Back => Self::Front,
            }
        }
    }
    impl From<Direction> for CubeSurface {
        fn from(value: Direction) -> Self {
            match value {
                Direction::Up => Self::Top,
                Direction::Down => Self::Bottom,
                Direction::Left => Self::Left,
                Direction::Right => Self::Right,
            }
        }
    }

    /// A potentially rotated position of a cube, where its original front
    /// surface may no longer be in the front position.
    #[derive(Debug)]
    struct CubePosition {
        /// The original surface currently in the front position.
        front: CubeSurface,
        /// The original surface currently in the left position.
        left: CubeSurface,
        /// The original surface currently in the top position.
        top: CubeSurface,
    }
    impl Default for CubePosition {
        fn default() -> Self {
            Self {
                front: CubeSurface::Front,
                top: CubeSurface::Top,
                left: CubeSurface::Left,
            }
        }
    }
    impl CubePosition {
        /// Returns the rotated cube such that the current front surface
        /// moves in the specified cardinal `direction`.
        pub fn rotate(&self, direction: Direction) -> Self {
            match direction {
                Direction::Up => Self {
                    front: self.top,
                    top: -self.front,
                    left: self.left,
                },
                Direction::Down => Self {
                    front: -self.top,
                    top: self.front,
                    left: self.left,
                },
                Direction::Left => Self {
                    front: self.left,
                    top: self.top,
                    left: -self.front,
                },
                Direction::Right => Self {
                    front: -self.left,
                    top: self.top,
                    left: self.front,
                },
            }
        }

        /// Returns the direction of an original surface relative to the current
        /// front, if it is in one of the cardinal direction from the
        /// current front.
        ///
        /// Therefore, [`None`] will be returned if the requested surface is
        /// currently in the front or back position.
        pub fn direction_of(&self, seeking: CubeSurface) -> Option<Direction> {
            if seeking == self.top {
                Some(Direction::Up)
            } else if seeking == -self.top {
                Some(Direction::Down)
            } else if seeking == self.left {
                Some(Direction::Left)
            } else if seeking == -self.left {
                Some(Direction::Right)
            } else {
                None
            }
        }
    }

    /// Coordinate system of the full, main grid per the standard [`Grid`]
    /// convention.
    pub struct MainGrid;

    /// Coordinate system of the smaller grid of only full tiles  per the
    /// standard [`Grid`] convention.
    struct TileGrid;

    /// Coordinate system of a grid for a local tile, relative to the
    /// upper left corner of the tile,  per the standard [`Grid`] convention.
    struct TileLocal;

    /// Converter between the various coordinate systems.
    struct GridTransformations(pub usize);
    impl GridTransformations {
        /// Converts a `point` in the main grid to its corresponding tile point.
        ///
        /// Note that this is a many-to-one function.
        pub fn main_to_tile(&self, point: GridPoint<MainGrid>) -> GridPoint<TileGrid> {
            (point / self.0).cast_unit()
        }

        /// Converts a `point` in the tile grid to the upper left coordinate of
        /// the tile in the main grid.
        ///
        /// Note that this is a one-to-one function.
        pub fn tile_to_main(&self, point: GridPoint<TileGrid>) -> GridPoint<MainGrid> {
            (point * self.0).cast_unit()
        }

        /// Converts a `point` in the main grid to the local grid for its tile.
        ///
        /// Note that this is a many-to-one function.
        pub fn main_to_local(&self, point: GridPoint<MainGrid>) -> GridPoint<TileLocal> {
            point.rem_euclid(&Size2D::new(self.0, self.0)).cast_unit()
        }

        /// Converts a `point` within a local tile to its point in the main grid
        /// given the position of the `tile` in the tile grid.
        ///
        /// Note that this is a one-to-one function.
        pub fn local_to_main(
            &self,
            tile: GridPoint<TileGrid>,
            point: GridPoint<TileLocal>,
        ) -> GridPoint<MainGrid> {
            self.tile_to_main(tile) + point.cast_unit().to_vector()
        }

        /// Returns the point on edge of a `destination` tile from an initial
        /// `position` on the edge of a source tile if we were to move forward
        /// one square and transition from the edge of the source tile to the
        /// edge of the destination tile.
        pub fn stitch_tiles(
            &self,
            position: &Position,
            destination: &CubeTile,
        ) -> GridPoint<MainGrid> {
            // The source point local to the source tile.
            let source_point_local = self.main_to_local(position.point);

            // Local distance along the edge of the source tile in the transverse coordinate
            let transverse_local = match position.facing {
                Direction::Up | Direction::Down => source_point_local.x,
                Direction::Left | Direction::Right => source_point_local.y,
            };
            // Inverse local distance along the edge of the source tile in the transverse
            // coordinate
            let transverse_local_inverse = self.0 - 1 - transverse_local;

            // The local position of the normal coordinate in the destination tile.
            let normal_dest = match destination.side {
                Direction::Up => 0,
                Direction::Down => self.0 - 1,
                Direction::Left => 0,
                Direction::Right => self.0 - 1,
            };

            let dest_point_local = match position.facing {
                Direction::Up | Direction::Right => match destination.side {
                    Direction::Up => Point2D::new(transverse_local_inverse, normal_dest),
                    Direction::Right => Point2D::new(normal_dest, transverse_local_inverse),
                    Direction::Down => Point2D::new(transverse_local, normal_dest),
                    Direction::Left => Point2D::new(normal_dest, transverse_local),
                },
                Direction::Down | Direction::Left => match destination.side {
                    Direction::Up => Point2D::new(transverse_local, normal_dest),
                    Direction::Right => Point2D::new(normal_dest, transverse_local),
                    Direction::Down => Point2D::new(transverse_local_inverse, normal_dest),
                    Direction::Left => Point2D::new(normal_dest, transverse_local_inverse),
                },
            };

            self.local_to_main(destination.tile, dest_point_local)
        }
    }

    /// A tile as though it were on the surface of a cube.
    #[derive(Debug, new)]
    struct CubeTile {
        /// The position of the tile in the tile grid.
        tile: GridPoint<TileGrid>,
        /// The direction we would either move off of or onto this tile.
        side: Direction,
    }

    /// Models a grid of tiles as the unfolded surface of a cube, allowing
    /// transitions between tiles with this interpretation.
    #[derive(new)]
    struct UnfoldedCube {
        /// The tile grid that includes the cube surfaces as well as void tiles.
        cube_grid: Grid<StdBool, TileGrid>,
    }
    impl UnfoldedCube {
        /// Returns the destination tile and the direction we would move onto it
        /// if we were to move off of `source_tile` in its direction.
        ///
        /// The directions are all relative to the overall tile grid as it is
        /// unfolded flat.
        pub fn lookup_destination_tile(&self, source_tile: &CubeTile) -> AocResult<CubeTile> {
            /// Internal structure for
            /// [`UnfoldedCube::lookup_destination_tile`].
            ///
            /// Holds the state for the tree search.
            struct LookupState {
                /// The cube surface that is being sought.
                seeking: CubeSurface,
                /// The cube tile, if the sought surface has been found.
                found: Option<CubeTile>,
                /// The set of cube tiles that have already been visited and
                /// checked.
                visited: HashSet<AnyGridPoint<TileGrid>>,
            }
            impl LookupState {
                /// Initializes a new state for the surface we are `seeking`.
                pub fn new(seeking: CubeSurface) -> Self {
                    Self {
                        seeking,
                        found: None,
                        visited: HashSet::new(),
                    }
                }
            }

            /// Internal structure for
            /// [`UnfoldedCube::lookup_destination_tile`].
            ///
            /// The tree node structure.
            struct LookupNode<'a> {
                /// The tile grid.
                grid: &'a Grid<StdBool, TileGrid>,
                /// The current tile in the tile grid.
                current: AnyGridPoint<TileGrid>,
                /// The model of the cube in its position for this tile.
                cube_model: CubePosition,
            }
            impl GlobalStateTreeNode for LookupNode<'_> {
                type GlobalState = LookupState;

                fn recurse_action(self, global_state: &mut Self::GlobalState) -> NodeAction<Self> {
                    // Have we already been here?
                    if global_state.visited.contains(&self.current) {
                        return NodeAction::Stop;
                    }

                    // Do we have a match?
                    if self.cube_model.front == global_state.seeking {
                        if global_state.found.is_some() {
                            // We've already found the tile, so this is a problem
                            global_state.found = None;
                            return NodeAction::Complete;
                        }
                        global_state.found = Some(CubeTile::new(
                            self.grid.bounded_point(&self.current).unwrap(),
                            self.cube_model.direction_of(CubeSurface::Front).unwrap(),
                        ));
                        return NodeAction::Stop;
                    }

                    // Note that we have visited this
                    global_state.visited.insert(self.current);

                    let children = Direction::iter()
                        .filter_map(|dir| {
                            let new_tile = self.current + dir.as_vector();

                            self.grid
                                .bounded_point(&new_tile)
                                .filter(|p| **self.grid.get(p))
                                .map(|_| Self {
                                    grid: self.grid,
                                    current: new_tile,
                                    cube_model: self.cube_model.rotate(dir),
                                })
                        })
                        .collect_vec();

                    if children.is_empty() {
                        NodeAction::Stop
                    } else {
                        NodeAction::Continue(children)
                    }
                }
            }

            LookupNode {
                grid: &self.cube_grid,
                current: source_tile.tile.try_cast().unwrap(),
                cube_model: CubePosition::default(),
            }
            .traverse_tree(LookupState::new(source_tile.side.into()))
            .found
            .ok_or(AocError::Process(
                "The grid is not a valid cube unfolding".into(),
            ))
        }
    }

    /// Behavior for part two.
    pub struct PartTwo<'a> {
        /// The main grid.
        grid: &'a Grid<Space, MainGrid>,
        /// The unfolded cube model for determining tile transitions.
        unfolded_cube: UnfoldedCube,
        /// Transformer between the various coordinate systems.
        transformation: GridTransformations,
    }
    impl<'a> Part<'a> for PartTwo<'a> {
        fn new(grid: &'a Grid<Space, MainGrid>) -> AocResult<Self> {
            let size = grid.size();

            // Determine the size of each cube face
            let face_size = if size.width.is_multiple_of(3) && size.height.is_multiple_of(4) {
                size.width / 3
            } else if size.width.is_multiple_of(4) && size.height.is_multiple_of(3) {
                size.width / 4
            } else {
                return Err(AocError::Process(
                    "Monkey map is evidently not a cube".into(),
                ));
            };

            let transformation = GridTransformations(face_size);
            let mut unfolded_grid = Grid::default((size / face_size).cast_unit());

            for unfolded_point in unfolded_grid.all_points() {
                if !grid
                    .get(&transformation.tile_to_main(unfolded_point))
                    .is_empty()
                {
                    unfolded_grid.set(&unfolded_point, true.into());
                }
            }

            Ok(Self {
                grid,
                unfolded_cube: UnfoldedCube::new(unfolded_grid),
                transformation,
            })
        }

        fn next_space(&self, position: &Position) -> AocResult<Position> {
            // If we are in a void, then follow our facing direction until we reach a real
            // tile
            if self.grid.get(&position.point).is_empty() {
                return PartOne::new(self.grid)?.next_space(position);
            }

            // The naive next point
            let new_point = position.point.try_cast().unwrap() + position.facing.as_vector();

            Ok(
                if self
                    .grid
                    .get_any(&new_point)
                    .copied()
                    .unwrap_or(Space::Void)
                    .is_empty()
                {
                    // Need to go to a new tile per the unfolded cube
                    let unfolded_point = self.transformation.main_to_tile(position.point);

                    let destination = self
                        .unfolded_cube
                        .lookup_destination_tile(&CubeTile::new(unfolded_point, position.facing))?;

                    Position::new(
                        self.transformation.stitch_tiles(position, &destination),
                        -destination.side,
                    )
                } else {
                    // We are still in the same tile
                    Position::new(new_point.try_cast().unwrap(), position.facing)
                },
            )
        }
    }

    /// A position on the main grid along with a direction we are facing.
    #[derive(Debug, Clone, new)]
    pub struct Position {
        /// The position on the main grid.
        point: GridPoint<MainGrid>,
        /// The direction we are facing on the main grid.
        facing: Direction,
    }
    impl Default for Position {
        fn default() -> Self {
            Self {
                point: Point2D::zero(),
                facing: Direction::Right,
            }
        }
    }
    impl Position {
        /// Calculates the password for this position.
        pub fn password(&self) -> u64 {
            let point = self.point.try_cast::<u64>().unwrap();

            1000 * (point.y + 1) + 4 * (point.x + 1) + self.facing.facing_value()
        }
    }

    /// [`Iterator`] over key points on the path for a given part of the
    /// problem.
    ///
    /// The yielded key positions are positions after we run into a wall,
    /// stop walking forward, or make a turn.
    ///
    /// If a problem occurs, an `Err` will be yielded and this will be the
    /// last item.
    ///
    /// The first item is the starting position.
    struct PathTraversal<'a, P> {
        /// The monkey map.
        monkey_map: &'a MonkeyMap,
        /// Holds data pertaining to the model for the part of the problem.
        part: P,
        /// Iterator over the steps in our path.
        path: Iter<'a, Step>,
        /// The next key position to be yielded.
        next_position: Option<Position>,
    }
    impl<'a, P: Part<'a>> Iterator for PathTraversal<'a, P> {
        type Item = AocResult<Position>;

        fn next(&mut self) -> Option<Self::Item> {
            let ret = self.next_position.clone();

            if let Some(ref pos) = ret {
                let res = self.path.next().map(|step| -> AocResult<_> {
                    match step {
                        Step::WalkForward(n) => {
                            let mut position = pos.clone();

                            for _ in 0..*n {
                                let new_position = self.part.next_space(&position)?;
                                match self.monkey_map.grid.get(&new_position.point) {
                                    Space::Void => panic!(),
                                    Space::Open => {
                                        position = new_position;
                                    }
                                    Space::Wall => break,
                                }
                            }

                            Ok(position)
                        }
                        Step::TurnLeft => Ok(Position::new(pos.point, pos.facing.turn_left())),
                        Step::TurnRight => Ok(Position::new(pos.point, pos.facing.turn_right())),
                    }
                });

                match res.transpose() {
                    Ok(o) => {
                        self.next_position = o;
                    }
                    Err(e) => {
                        self.next_position = None;
                        return Some(Err(e));
                    }
                }
            }

            ret.map(Ok)
        }
    }
    impl<'a, P: Part<'a>> FusedIterator for PathTraversal<'a, P> {}
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 22,
    name: "Monkey Map",
    preprocessor: Some(|input| Ok(Box::new(MonkeyMap::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<MonkeyMap>()?
                .password::<PartOne>()?
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<MonkeyMap>()?
                .password::<PartTwo>()?
                .into())
        },
    ],
};
