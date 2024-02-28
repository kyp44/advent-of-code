use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = ">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>";
            answers = unsigned![3068, 1514285714288];
        }
        actual_answers = unsigned![3184];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::grid::StdBool;
    use circular_buffer::CircularBuffer;
    use euclid::{Length, Point2D, Vector2D};
    use itertools::Itertools;
    use num::integer::lcm;
    use std::collections::HashSet;

    mod rock_shapes {
        use super::*;
        use euclid::point2;

        pub const LINE_HORIZONTAL: &[Point<RockSpace>] =
            &[point2(0, 0), point2(1, 0), point2(2, 0), point2(3, 0)];
        pub const PLUS: &[Point<RockSpace>] = &[
            point2(1, 0),
            point2(0, 1),
            point2(1, 1),
            point2(2, 1),
            point2(1, 2),
        ];
        pub const RIGHT_ANGLE: &[Point<RockSpace>] = &[
            point2(0, 0),
            point2(1, 0),
            point2(2, 0),
            point2(2, 1),
            point2(2, 2),
        ];
        pub const LINE_VERTICAL: &[Point<RockSpace>] =
            &[point2(0, 0), point2(0, 1), point2(0, 2), point2(0, 3)];
        pub const SQUARE: &[Point<RockSpace>] =
            &[point2(0, 0), point2(1, 0), point2(0, 1), point2(1, 1)];
    }

    struct RockSpace;
    struct ChamberRelativeSpace;
    struct ChamberAbsoluteSpace;

    const NUM_ROCKS: usize = 5;
    const CHAMBER_WIDTH: isize = 7;
    const BUFFER_SIZE: usize = 10;
    // From left side of chamber
    const ROCK_SPAWN_DX: isize = 2;
    /// From height of tower
    const ROCK_SPAWN_DY: isize = 3;

    type Point<U> = Point2D<isize, U>;
    type Vector<U> = Vector2D<isize, U>;

    #[derive(Debug, Clone, Copy)]
    enum JetDirection {
        Left,
        Right,
    }
    impl TryFrom<char> for JetDirection {
        type Error = AocError;

        fn try_from(value: char) -> Result<Self, Self::Error> {
            Ok(match value {
                '<' => Self::Left,
                '>' => Self::Right,
                _ => {
                    return Err(AocError::InvalidInput(
                        format!("'{value}' is not a valid jet direction").into(),
                    ))
                }
            })
        }
    }
    impl JetDirection {
        pub fn direction_vector<U>(&self) -> Vector<U> {
            match self {
                JetDirection::Left => Vector::new(-1, 0),
                JetDirection::Right => Vector::new(1, 0),
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum RockType {
        LineHorizontal,
        Plus,
        RightAngle,
        LineVertical,
        Square,
    }
    impl RockType {
        pub fn points(&self) -> &'static [Point<RockSpace>] {
            match self {
                RockType::LineHorizontal => &rock_shapes::LINE_HORIZONTAL,
                RockType::Plus => &rock_shapes::PLUS,
                RockType::RightAngle => &rock_shapes::RIGHT_ANGLE,
                RockType::LineVertical => &rock_shapes::LINE_VERTICAL,
                RockType::Square => &rock_shapes::SQUARE,
            }
        }
    }

    #[derive(Debug, Clone)]
    struct Rock {
        rock_type: RockType,
        lower_left: Point<ChamberRelativeSpace>,
    }
    impl Rock {
        // TODO: Can we just create at a position?
        /* pub fn move_lower_left_to(&self, point: Point) -> Option<Self> {
            self.move_relative(point.to_vec())
        } */

        pub fn move_relative(&self, direction: Vector<ChamberRelativeSpace>) -> Option<Self> {
            let lower_left = self.lower_left + direction;

            // TODO: Do we need to better detect negative height to panic? Or maybe just outside this?
            ((0..CHAMBER_WIDTH).contains(&lower_left.x) && lower_left.y >= 0).then(|| Self {
                rock_type: self.rock_type,
                lower_left,
            })
        }

        pub fn collides(&self, other: &Self) -> bool {}
    }

    #[derive(Default)]
    struct ChamberRocks {
        fallen_rocks: CircularBuffer<BUFFER_SIZE, Rock>,
        floor_height: Length<u64, ChamberAbsoluteSpace>,
        tower_height: Length<u64, ChamberRelativeSpace>,
    }
    impl ChamberRocks {
        pub fn tower_height(&self) -> u64 {
            self.floor_height + u64::try_from(self.relative_tower_height).unwrap()
        }

        pub fn collides(&self, rock: &Rock) -> bool {
            !rock.points.is_disjoint(&self.fallen_rocks)
        }

        // Adds no matter what without checking for collisions.
        pub fn add_rock(&mut self, rock: Rock) {
            self.fallen_rocks.extend(rock.points);

            self.remove_old();
        }

        // Optimization for part two
        fn remove_old(&mut self) {
            let height = self.tower_height();

            self.fallen_rocks.retain(|p| p.y > height - 50);
        }
    }
    impl std::fmt::Debug for ChamberRocks {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let points = self
                .fallen_rocks
                .iter()
                .map(|p| Point::new(p.x, -p.y))
                .collect_vec();
            let grid: Grid<StdBool> = Grid::from_coordinates(points.iter());

            write!(f, "{grid:?}")
        }
    }

    #[derive(Debug)]
    pub struct Chamber {
        jet_directions: Vec<JetDirection>,
    }
    impl FromStr for Chamber {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let jet_directions = s
                .trim()
                .chars()
                .map(|c| JetDirection::try_from(c))
                .collect::<Result<Vec<_>, _>>()?;

            if jet_directions.is_empty() {
                return Err(AocError::InvalidInput(
                    "No jet directions were specified!".into(),
                ));
            }

            Ok(Self { jet_directions })
        }
    }
    impl Chamber {
        /// Simulates the falling rocks and returns the tower height after
        /// `num_rocks` have fallen.
        pub fn simulate(&self, num_rocks: usize) -> ChamberRocks {
            let mut jet_directions = self.jet_directions.iter().cycle();
            let mut chamber_rocks = ChamberRocks::default();

            for rock in self.rock_types.iter().cycle().take(num_rocks) {
                // Spawn in rock
                let mut rock = rock
                    .move_lower_left_to(Point::new(
                        ROCK_SPAWN_DX,
                        chamber_rocks.tower_height() + ROCK_SPAWN_DY,
                    ))
                    .unwrap();

                loop {
                    // Push with jet if possible
                    if let Some(r) =
                        rock.move_relative(jet_directions.next().unwrap().direction_vector())
                    {
                        if !chamber_rocks.collides(&r) {
                            rock = r;
                        }
                    }

                    // Move down if possible
                    match rock.move_relative(-Vector::unit_y()) {
                        Some(r) => {
                            if chamber_rocks.collides(&r) {
                                // Lands at old position
                                chamber_rocks.add_rock(rock);
                                break;
                            } else {
                                // Keep on moving
                                rock = r;
                            }
                        }
                        None => {
                            // Lands on the floor
                            chamber_rocks.add_rock(rock);
                            break;
                        }
                    }
                }
            }

            chamber_rocks
        }

        pub fn tower_height(&self, num_rocks: usize) -> u64 {
            /* let lcm = lcm(self.rock_types.len(), self.jet_directions.len());

            for m in 1..=5 {
                let chamber_rocks = self.simulate(m * lcm);

                println!(
                    "TODO: {m}: {}\n{chamber_rocks:?}\n",
                    chamber_rocks.tower_height(),
                );
            } */
            self.simulate(num_rocks).tower_height().try_into().unwrap()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 17,
    name: "Pyroclastic Flow",
    preprocessor: Some(|input| Ok(Box::new(Chamber::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<Chamber>()?.tower_height(2022).into())
        },
        // Part two
        /* |input| {
            // Process
            Ok(input
                .expect_data::<Chamber>()?
                .tower_height(1000000000000)
                .into())
        }, */
    ],
};
