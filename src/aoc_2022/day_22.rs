use std::str::FromStr;

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
        actual_answers = unsigned![103224];
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
    use enum_map::{Enum, EnumMap};
    use euclid::{Length, Point2D, Vector2D};
    use itertools::Itertools;
    use nom::{branch::alt, bytes::complete::tag, combinator::map, multi::many1};
    use petgraph::graph::Node;
    use strum::{Display, EnumIter, IntoEnumIterator};

    use super::*;

    #[derive(Clone, Copy, Default, PartialEq, Eq)]
    pub enum Tile {
        #[default]
        Void,
        Open,
        Wall,
    }
    impl Tile {
        pub fn is_empty(&self) -> bool {
            self == &Self::Void
        }
    }
    impl TryFrom<char> for Tile {
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
    impl std::fmt::Debug for Tile {
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

    #[derive(Debug)]
    enum Step {
        WalkForward(u8),
        TurnLeft,
        TurnRight,
    }
    impl Parsable<'_> for Step {
        fn parser(input: &'_ str) -> NomParseResult<&str, Self> {
            alt((
                map(nom::character::complete::u8, |n| Self::WalkForward(n)),
                map(tag("L"), |_| Self::TurnLeft),
                map(tag("R"), |_| Self::TurnRight),
            ))(input)
        }
    }

    struct ParsePath(Vec<Step>);
    impl Parsable<'_> for ParsePath {
        fn parser(input: &'_ str) -> NomParseResult<&str, Self> {
            map(many1(Step::parser), ParsePath)(input)
        }
    }

    #[derive(Debug, Clone, Copy, EnumIter)]
    pub enum Direction {
        Up,
        Down,
        Left,
        Right,
    }
    impl Direction {
        pub fn as_vector<U>(&self) -> Vector2D<isize, U> {
            match self {
                Direction::Up => -Vector2D::<isize, U>::unit_y(),
                Direction::Down => Vector2D::unit_y(),
                Direction::Left => -Vector2D::<isize, U>::unit_x(),
                Direction::Right => Vector2D::unit_x(),
            }
        }

        pub fn turn_left(&self) -> Self {
            match self {
                Direction::Up => Self::Left,
                Direction::Down => Self::Right,
                Direction::Left => Self::Down,
                Direction::Right => Self::Up,
            }
        }

        pub fn turn_right(&self) -> Self {
            match self {
                Direction::Up => Self::Right,
                Direction::Down => Self::Left,
                Direction::Left => Self::Up,
                Direction::Right => Self::Down,
            }
        }

        pub fn facing_value(&self) -> u64 {
            match self {
                Direction::Up => 3,
                Direction::Down => 1,
                Direction::Left => 2,
                Direction::Right => 0,
            }
        }
    }

    pub struct MonkeyMap {
        grid: Grid<Tile>,
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
        fn path_traversal<'a, P: Part<'a>>(&'a self) -> AocResult<PathTraversal<'a, P>> {
            let part = P::new(&self.grid)?;
            let position = part.next_space(&Position::new(GridPoint::zero(), Direction::Right))?;

            if *self.grid.get(&position.point) != Tile::Open {
                return Err(AocError::Process("The initial position is not open".into()));
            }

            Ok(PathTraversal {
                part,
                monkey_map: self,
                path: self.path.iter(),
                next_position: Some(position),
            })
        }

        pub fn password<'a, P: Part<'a>>(&'a self) -> AocResult<u64> {
            self.path_traversal::<P>()?
                .last()
                .ok_or(AocError::NoSolution)?
                .map(|p| p.password())
        }
    }

    pub trait Part<'a>: Sized {
        fn new(grid: &'a Grid<Tile>) -> AocResult<Self>;
        // Returns the position if we were to try to walk forward a single space,
        // moving ahead to the next tile if necessary.
        fn next_space(&self, position: &Position) -> AocResult<Position>;
    }

    pub struct PartOne<'a> {
        grid: &'a Grid<Tile>,
    }
    impl<'a> Part<'a> for PartOne<'a> {
        fn new(grid: &'a Grid<Tile>) -> AocResult<Self> {
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

    #[derive(Debug, Clone, Copy, Display, PartialEq, Eq)]
    enum CubeSurface {
        Front,
        Up,
        Right,
        Left,
        Down,
        Back,
    }
    impl Neg for CubeSurface {
        type Output = Self;

        fn neg(self) -> Self::Output {
            match self {
                CubeSurface::Front => Self::Back,
                CubeSurface::Up => Self::Down,
                CubeSurface::Right => Self::Left,
                CubeSurface::Left => Self::Right,
                CubeSurface::Down => Self::Up,
                CubeSurface::Back => Self::Front,
            }
        }
    }
    impl From<Direction> for CubeSurface {
        fn from(value: Direction) -> Self {
            match value {
                Direction::Up => Self::Up,
                Direction::Down => Self::Down,
                Direction::Left => Self::Left,
                Direction::Right => Self::Right,
            }
        }
    }

    #[derive(Debug)]
    struct CubePosition {
        front: CubeSurface,
        left: CubeSurface,
        up: CubeSurface,
    }
    impl Default for CubePosition {
        fn default() -> Self {
            Self {
                front: CubeSurface::Front,
                up: CubeSurface::Up,
                left: CubeSurface::Left,
            }
        }
    }
    impl CubePosition {
        pub fn rotate(&self, direction: Direction) -> Self {
            match direction {
                Direction::Up => Self {
                    front: self.up,
                    up: -self.front,
                    left: self.left,
                },
                Direction::Down => Self {
                    front: -self.up,
                    up: self.front,
                    left: self.left,
                },
                Direction::Left => Self {
                    front: self.left,
                    up: self.up,
                    left: -self.front,
                },
                Direction::Right => Self {
                    front: -self.left,
                    up: self.up,
                    left: self.front,
                },
            }
        }

        // The direction of a surface relative to the front, None if the surface is in back or front.
        pub fn drection_of(&self, seeking: CubeSurface) -> Option<Direction> {
            if seeking == self.up {
                Some(Direction::Up)
            } else if seeking == -self.up {
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

    // Coord system of local to tile
    struct TileLocal;

    struct GridTransformations(pub usize);
    impl GridTransformations {
        pub fn grid_to_unfolded(&self, point: GridPoint) -> GridPoint<UnfoldedCube> {
            (point / self.0).cast_unit()
        }

        // To upper left of tile
        pub fn unfolded_to_grid(&self, point: GridPoint<UnfoldedCube>) -> GridPoint {
            (point * self.0).cast_unit()
        }

        pub fn stitch_tiles(
            &self,
            source: &CubeTile,
            distance: Length<usize, TileLocal>,
            destination: &CubeTile,
        ) -> GridPoint<GridSpace> {
            match source.side {
                Direction::Up | Direction::Right => match destination.side {
                    Direction::Up => todo!(),
                    Direction::Down => todo!(),
                    Direction::Left => todo!(),
                    Direction::Right => todo!(),
                },
                Direction::Down | Direction::Left => match destination.side {
                    Direction::Up => todo!(),
                    Direction::Down => todo!(),
                    Direction::Left => todo!(),
                    Direction::Right => todo!(),
                },
            }
        }
    }

    #[derive(Debug, new)]
    struct CubeTile {
        tile: GridPoint<UnfoldedCube>,
        side: Direction,
    }

    #[derive(new)]
    struct UnfoldedCube {
        cube_grid: Grid<StdBool, Self>,
    }
    impl UnfoldedCube {
        pub fn lookup_destination_tile(&self, source_tile: &CubeTile) -> AocResult<CubeTile> {
            struct LookupState {
                seeking: CubeSurface,
                solution: Option<CubeTile>,
                visited: HashSet<AnyGridPoint<UnfoldedCube>>,
            }
            impl LookupState {
                pub fn new(seeking: CubeSurface) -> Self {
                    Self {
                        seeking,
                        solution: None,
                        visited: HashSet::new(),
                    }
                }
            }

            struct LookupNode<'a> {
                grid: &'a Grid<StdBool, UnfoldedCube>,
                current: AnyGridPoint<UnfoldedCube>,
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
                        if global_state.solution.is_some() {
                            // We've already found the tile, so this is a problem
                            global_state.solution = None;
                            return NodeAction::Complete;
                        }
                        global_state.solution = Some(CubeTile::new(
                            self.grid.bounded_point(&self.current).unwrap(),
                            self.cube_model.drection_of(CubeSurface::Front).unwrap(),
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
            .solution
            .ok_or(AocError::Process(
                "The grid is not a valid cube unfolding".into(),
            ))
        }
    }

    pub struct PartTwo<'a> {
        grid: &'a Grid<Tile>,
        unfolded_cube: UnfoldedCube,
        transformation: GridTransformations,
    }
    impl<'a> Part<'a> for PartTwo<'a> {
        fn new(grid: &'a Grid<Tile>) -> AocResult<Self> {
            let size = grid.size();

            // Determine the size of each cube face
            let face_size = if size.width % 3 == 0 && size.height % 4 == 0 {
                size.width / 3
            } else if size.width % 4 == 0 && size.height % 3 == 0 {
                size.width / 4
            } else {
                return Err(AocError::Process(
                    "Monkey map is evidently not a cube".into(),
                ));
            };

            let transformation = GridTransformations(face_size);
            let mut unfolded_grid = Grid::default((*size / face_size).cast_unit());

            for unfolded_point in unfolded_grid.all_points() {
                if !grid
                    .get(&transformation.unfolded_to_grid(unfolded_point))
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
            // If we are in a void, then follow our facing direction until we reach a real tile
            if self.grid.get(&position.point).is_empty() {
                return PartOne::new(&self.grid)?.next_space(position);
            }

            // TODO: These may not be needed, can just put in expression.
            let point = position.point.try_cast().unwrap();
            let dir_vec = position.facing.as_vector();

            let new_point = point + dir_vec;

            Ok(
                if self.grid.get_any(&new_point).copied().unwrap_or(Tile::Void) != Tile::Void {
                    // We are still in the same tile
                    Position::new(new_point.try_cast().unwrap(), position.facing)
                } else {
                    // Need to go to a new tile per the unfolded cube
                    println!("\nTODO edge of tile at {position:?}");
                    let unfolded_point = self.transformation.grid_to_unfolded(position.point);
                    println!("TODO Unfolded point: {unfolded_point:?}");

                    let x = self
                        .unfolded_cube
                        .lookup_destination_tile(&CubeTile::new(unfolded_point, position.facing))?;
                    println!("TODO Dest tile: {x:?}");

                    PartOne::new(&self.grid).unwrap().next_space(position)?
                },
            )
        }
    }

    #[derive(Debug, Clone, new)]
    pub struct Position {
        point: GridPoint,
        facing: Direction,
    }
    impl Position {
        pub fn default(point: GridPoint) -> Self {
            Self {
                point,
                facing: Direction::Right,
            }
        }

        pub fn password(&self) -> u64 {
            let point = self.point.try_cast::<u64>().unwrap();

            1000 * (point.y + 1) + 4 * (point.x + 1) + self.facing.facing_value()
        }
    }

    // First yield is the starting position.
    struct PathTraversal<'a, P> {
        monkey_map: &'a MonkeyMap,
        part: P,
        path: Iter<'a, Step>,
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
                                    Tile::Void => panic!(),
                                    Tile::Open => {
                                        position = new_position;
                                    }
                                    Tile::Wall => break,
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
