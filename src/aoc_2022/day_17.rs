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
    use derive_new::new;
    use euclid::{point2, size2, vec2, Box2D, Length, Point2D, Size2D, Vector2D};
    use gat_lending_iterator::LendingIterator;
    use itertools::Itertools;
    use num::integer::lcm;
    use std::collections::HashSet;
    use strum::IntoEnumIterator;
    use strum_macros::EnumIter;

    mod rock_shapes {
        use super::*;

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
    // From height of tower
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
                JetDirection::Left => vec2(-1, 0),
                JetDirection::Right => vec2(1, 0),
            }
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EnumIter)]
    enum RockType {
        #[default]
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

        pub fn size(&self) -> Size2D<isize, RockSpace> {
            match self {
                RockType::LineHorizontal => size2(4, 1),
                RockType::Plus => size2(3, 3),
                RockType::RightAngle => size2(3, 3),
                RockType::LineVertical => size2(1, 4),
                RockType::Square => size2(2, 2),
            }
        }
    }

    // A rock in the chamber
    #[derive(Debug, Clone, PartialEq, Eq, new)]
    struct Rock {
        rock_type: RockType,
        lower_left: Point<ChamberRelativeSpace>,
    }
    impl Rock {
        // TODO: Can we just create at a position?
        /* pub fn move_lower_left_to(&self, point: Point) -> Option<Self> {
            self.move_relative(point.to_vec())
        } */
        fn points(&self) -> HashSet<Point<ChamberRelativeSpace>> {
            self.rock_type
                .points()
                .iter()
                .map(|p| self.lower_left + p.to_vector().cast_unit())
                .collect()
        }

        fn bounding_box(&self) -> Box2D<isize, ChamberRelativeSpace> {
            Box2D::from_origin_and_size(self.lower_left, self.rock_type.size().cast_unit())
        }

        pub fn collides(&self, other: &Self) -> bool {
            if self.bounding_box().intersects(&other.bounding_box()) {
                !self.points().is_disjoint(&other.points())
            } else {
                false
            }
        }
    }
    impl std::ops::Add<Vector<ChamberRelativeSpace>> for &Rock {
        type Output = Rock;

        fn add(self, rhs: Vector<ChamberRelativeSpace>) -> Self::Output {
            Rock {
                rock_type: self.rock_type,
                lower_left: self.lower_left + rhs,
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    enum CheckRock {
        Good,
        OutOfBounds,
        RockCollision,
        FallOutBottom,
    }

    #[derive(Default, Eq)]
    struct ChamberRocks {
        fallen_rocks: CircularBuffer<BUFFER_SIZE, Rock>,
        floor_height: Length<u64, ChamberAbsoluteSpace>,
        tower_height: Length<isize, ChamberRelativeSpace>,
        last_rock_type: RockType,
        last_jet_direction_idx: usize,
    }
    impl PartialEq for ChamberRocks {
        fn eq(&self, other: &Self) -> bool {
            self.fallen_rocks == other.fallen_rocks
                && self.last_rock_type == other.last_rock_type
                && self.last_jet_direction_idx == other.last_jet_direction_idx
        }
    }
    impl ChamberRocks {
        pub fn tower_height(&self) -> Length<u64, ChamberAbsoluteSpace> {
            self.floor_height + self.tower_height.cast_unit().try_cast().unwrap()
        }

        pub fn check_rock(&self, rock: &Rock) -> CheckRock {
            let point = rock.lower_left;

            if point.y < 0 {
                CheckRock::FallOutBottom
            } else {
                let rock_box = rock.bounding_box();
                let chamber_box = Box2D::new(
                    point2(0, rock_box.min.y),
                    point2(CHAMBER_WIDTH, rock_box.max.y),
                );
                if !chamber_box.contains_box(&rock_box) {
                    CheckRock::OutOfBounds
                } else {
                    if rock.lower_left.y <= self.tower_height.0
                        && self.fallen_rocks.iter().any(|r| rock.collides(r))
                    {
                        CheckRock::RockCollision
                    } else {
                        CheckRock::Good
                    }
                }
            }
        }

        // Adds no matter what without checking for collisions.
        pub fn add_rock(&mut self, rock: Rock, last_jet_direction_idx: usize) {
            self.last_rock_type = rock.rock_type;
            self.last_jet_direction_idx = last_jet_direction_idx;
            self.fallen_rocks.push_front(rock);

            // A rock was removed at the end so re-adjust floor and height.
            let mut floor_offset = isize::MAX;
            let mut height = 0;
            for rock in self.fallen_rocks.iter() {
                floor_offset = floor_offset.min(rock.lower_left.y);
                height = height.max(rock.bounding_box().max.y.try_into().unwrap());
            }
            self.floor_height += Length::new(floor_offset.try_into().unwrap());
            self.tower_height = Length::new(height - floor_offset);

            // Now adjust all rock locations since the new floor is at relative zero height.
            let offset = vec2(0, -floor_offset);
            for rock in self.fallen_rocks.iter_mut() {
                rock.lower_left += offset;
            }
        }
    }
    impl std::fmt::Debug for ChamberRocks {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let points = self
                .fallen_rocks
                .iter()
                .flat_map(|r| r.points().into_iter().map(|p| AnyGridPoint::new(p.x, -p.y)))
                .collect_vec();
            let grid: Grid<StdBool> = Grid::from_coordinates(points.iter());

            write!(f, "{grid:?}")
        }
    }

    struct ChamberSimulation<I> {
        jet_direction_iter: I,
        rock_type_iter: RockTypeIter,
        chamber_rocks: ChamberRocks,
    }
    impl<'a>
        ChamberSimulation<
            std::iter::Cycle<std::iter::Enumerate<std::slice::Iter<'a, JetDirection>>>,
        >
    {
        pub fn new(jet_directions: &'a [JetDirection]) -> Self {
            Self {
                jet_direction_iter: jet_directions.iter().enumerate().cycle(),
                rock_type_iter: RockType::iter(),
                chamber_rocks: ChamberRocks::default(),
            }
        }
    }
    impl<I: Iterator<Item = (usize, JetDirection)>> LendingIterator for ChamberSimulation<I> {
        type Item<'a> = &'a ChamberRocks
        where
            Self: 'a;

        fn next(&mut self) -> Option<Self::Item<'_>> {
            let rock_type = self.rock_type_iter.next().unwrap();

            // Spawn in rock
            let mut rock = Rock::new(
                rock_type,
                point2(
                    ROCK_SPAWN_DX,
                    self.chamber_rocks.tower_height.0 + ROCK_SPAWN_DY,
                ),
            );

            loop {
                // Push with jet if possible
                let (jet_direction_idx, jet_direction) = self.jet_direction_iter.next().unwrap();
                let new_rock = &rock + jet_direction.direction_vector();
                if let CheckRock::Good = self.chamber_rocks.check_rock(&new_rock) {
                    rock = new_rock;
                }

                // Move down if possible
                let new_rock = &rock + vec2(0, -1);
                match self.chamber_rocks.check_rock(&new_rock) {
                    CheckRock::Good => rock = new_rock,
                    CheckRock::FallOutBottom => {
                        if self.chamber_rocks.floor_height.0 == 0 {
                            self.chamber_rocks.add_rock(rock, jet_direction_idx);
                        }

                        // If a rock falls out the bottom of our current shifted buffer then, oh well,
                        // it contributes nothing.

                        break;
                    }
                    CheckRock::RockCollision => {
                        self.chamber_rocks.add_rock(rock, jet_direction_idx);
                        break;
                    }
                    _ => panic!(),
                }
            }

            Some(&self.chamber_rocks)
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
        pub fn tower_height(&self, num_rocks: usize) -> u64 {
            let mut simulation = ChamberSimulation::new(&self.jet_directions);
            /* let lcm = lcm(self.rock_types.len(), self.jet_directions.len());

            for m in 1..=5 {
                let chamber_rocks = self.simulate(m * lcm);

                println!(
                    "TODO: {m}: {}\n{chamber_rocks:?}\n",
                    chamber_rocks.tower_height(),
                );
            } */
            simulation.next()
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
