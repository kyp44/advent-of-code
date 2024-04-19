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
    use std::{slice::Iter, str::FromStr};

    use derive_new::new;
    use euclid::Vector2D;
    use nom::{branch::alt, bytes::complete::tag, combinator::map, multi::many1};

    use super::*;

    #[derive(Clone, Copy, Default, PartialEq, Eq)]
    pub enum Tile {
        #[default]
        Void,
        Open,
        Wall,
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

    #[derive(Debug, Clone, Copy)]
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
            let position = part.next_tile(GridPoint::zero(), Direction::Right);

            if *self.grid.get(&position) != Tile::Open {
                return Err(AocError::Process("The initial position is not open".into()));
            }

            Ok(PathTraversal {
                part,
                monkey_map: self,
                path: self.path.iter(),
                next_position: Some(Position::default(position)),
            })
        }

        pub fn password<'a, P: Part<'a>>(&'a self) -> AocResult<u64> {
            self.path_traversal::<P>()?
                .last()
                .ok_or(AocError::NoSolution)
                .map(|p| p.password())
        }
    }

    pub trait Part<'a>: Sized {
        fn new(grid: &'a Grid<Tile>) -> AocResult<Self>;
        fn next_tile(&self, point: GridPoint, direction: Direction) -> GridPoint;
    }

    pub struct PartOne<'a> {
        grid: &'a Grid<Tile>,
    }
    impl<'a> Part<'a> for PartOne<'a> {
        fn new(grid: &'a Grid<Tile>) -> AocResult<Self> {
            Ok(Self { grid })
        }

        fn next_tile(&self, point: GridPoint, direction: Direction) -> GridPoint {
            let mut point = point.try_cast().unwrap();
            let size = self.grid.size().try_cast().unwrap();
            let dir_vec = direction.as_vector();

            loop {
                point = (point + dir_vec).rem_euclid(&size);

                if *self.grid.get_any(&point).unwrap() != Tile::Void {
                    break point.try_cast().unwrap();
                }
            }
        }
    }

    pub struct PartTwo<'a> {
        grid: &'a Grid<Tile>,
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

            println!("TODO: {face_size}");

            Ok(Self { grid })
        }

        fn next_tile(&self, point: GridPoint, direction: Direction) -> GridPoint {
            PartOne::new(&self.grid)
                .unwrap()
                .next_tile(point, direction)
        }
    }

    #[derive(Debug, Clone, new)]
    struct Position {
        position: GridPoint,
        facing: Direction,
    }
    impl Position {
        pub fn default(position: GridPoint) -> Self {
            Self {
                position,
                facing: Direction::Right,
            }
        }

        pub fn password(&self) -> u64 {
            let pos = self.position.try_cast::<u64>().unwrap();

            1000 * (pos.y + 1) + 4 * (pos.x + 1) + self.facing.facing_value()
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
        type Item = Position;

        fn next(&mut self) -> Option<Self::Item> {
            let ret = self.next_position.clone();

            if let Some(ref pos) = ret {
                self.next_position = self.path.next().map(|step| match step {
                    Step::WalkForward(n) => {
                        let mut point = pos.position;

                        for _ in 0..*n {
                            let new_point = self.part.next_tile(point, pos.facing);
                            match self.monkey_map.grid.get(&new_point) {
                                Tile::Void => panic!(),
                                Tile::Open => {
                                    point = new_point;
                                }
                                Tile::Wall => break,
                            }
                        }

                        Position::new(point, pos.facing)
                    }
                    Step::TurnLeft => Position::new(pos.position, pos.facing.turn_left()),
                    Step::TurnRight => Position::new(pos.position, pos.facing.turn_right()),
                });
            }

            ret
        }
    }
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
