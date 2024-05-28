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
        actual_answers = unsigned![3184, 1577077363915];
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
    use num::{integer::lcm, Integer};
    use std::collections::HashSet;
    use strum::{EnumCount, EnumIter, IntoEnumIterator};

    /// Rock shape constants.
    mod rock_shapes {
        use super::*;

        /// Relative points for the horizontal line rock shape.
        pub const LINE_HORIZONTAL: &[Point<RockSpace>] =
            &[point2(0, 0), point2(1, 0), point2(2, 0), point2(3, 0)];
        /// Relative points for the plus rock shape.
        pub const PLUS: &[Point<RockSpace>] = &[
            point2(1, 0),
            point2(0, 1),
            point2(1, 1),
            point2(2, 1),
            point2(1, 2),
        ];
        /// Relative points for the right angle rock shape.
        pub const RIGHT_ANGLE: &[Point<RockSpace>] = &[
            point2(0, 0),
            point2(1, 0),
            point2(2, 0),
            point2(2, 1),
            point2(2, 2),
        ];
        /// Relative points for the vertical line rock shape.
        pub const LINE_VERTICAL: &[Point<RockSpace>] =
            &[point2(0, 0), point2(0, 1), point2(0, 2), point2(0, 3)];
        /// Relative points for the square rock shape.
        pub const SQUARE: &[Point<RockSpace>] =
            &[point2(0, 0), point2(1, 0), point2(0, 1), point2(1, 1)];
    }

    /// The coordinate space relative to the lower left corner of a rock.
    struct RockSpace;
    /// The coordinate space relative to the lower left of the current chamber buffer,
    /// where `y = 0` is the height of the current buffer floor.
    struct ChamberRelativeSpace;
    /// The coordinate space relative to the lower left of the overall chamber,
    /// where `y = 0` is absolute floor of the chamber.
    struct ChamberAbsoluteSpace;

    /// The width of the chamber.
    const CHAMBER_WIDTH: isize = 7;
    /// The number of rocks to keep in chamber the circular buffer.
    ///
    /// NOTE: 10 is not enough to yield the correct answer in all cases.
    const BUFFER_SIZE: usize = 20;
    /// The `x` location to spawn new rocks in relative to the left of the chamber.
    const ROCK_SPAWN_DX: isize = 2;
    /// The `y` location to spawn new rocks in relative to current height of the
    /// chamber tower.
    const ROCK_SPAWN_DY: isize = 3;

    /// Chamber and rock 2D points in a particular coordinate space `U`.
    type Point<U> = Point2D<isize, U>;
    /// Chamber and rock 2D vectors in a particular coordinate space `U`.
    type Vector<U> = Vector2D<isize, U>;

    /// The direction of a jet of hot gas that pushes falling rocks, which
    /// can be parsed from text input.
    #[derive(Debug, Clone, Copy)]
    enum JetDirection {
        /// Pushes rocks to the left.
        Left,
        /// Pushes rocks to the right.
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
        /// Returns the displacement vector for this direction in coordinate space `U`.
        pub fn direction_vector<U>(&self) -> Vector<U> {
            match self {
                JetDirection::Left => vec2(-1, 0),
                JetDirection::Right => vec2(1, 0),
            }
        }
    }

    /// The different rock shapes.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EnumIter, EnumCount)]
    enum RockShape {
        /// Horizontal line.
        /// ```[text]
        /// ####
        /// ```
        #[default]
        LineHorizontal,
        /// Plus.
        /// ```[text]
        /// .#.
        /// ###
        /// .#.
        /// ```
        Plus,
        /// Right angle.
        /// ```[text]
        /// ..#
        /// ..#
        /// ###
        /// ```
        RightAngle,
        /// Vertical line.
        /// ```[text]
        /// #
        /// #
        /// #
        /// #
        /// ```
        LineVertical,
        /// Square.
        /// ```[text]
        /// ##
        /// ##
        /// ```
        Square,
    }
    impl RockShape {
        /// Returns a list of points that make up the shape of the rock shape
        /// in [`RockSpace`].
        pub fn points(&self) -> &'static [Point<RockSpace>] {
            match self {
                RockShape::LineHorizontal => rock_shapes::LINE_HORIZONTAL,
                RockShape::Plus => rock_shapes::PLUS,
                RockShape::RightAngle => rock_shapes::RIGHT_ANGLE,
                RockShape::LineVertical => rock_shapes::LINE_VERTICAL,
                RockShape::Square => rock_shapes::SQUARE,
            }
        }

        /// Returns the size of the rock shape.
        pub fn size(&self) -> Size2D<isize, RockSpace> {
            match self {
                RockShape::LineHorizontal => size2(4, 1),
                RockShape::Plus => size2(3, 3),
                RockShape::RightAngle => size2(3, 3),
                RockShape::LineVertical => size2(1, 4),
                RockShape::Square => size2(2, 2),
            }
        }
    }

    /// A rock located in the chamber.
    #[derive(Debug, Clone, PartialEq, Eq, new)]
    struct Rock {
        /// The shape of the rock.
        rock_shape: RockShape,
        /// The location of the lower left corner of the rock in the chamber buffer.
        lower_left: Point<ChamberRelativeSpace>,
    }
    impl Rock {
        /// Returns the set of points that make up the rock in [`ChamberRelativeSpace`].
        fn points(&self) -> HashSet<Point<ChamberRelativeSpace>> {
            self.rock_shape
                .points()
                .iter()
                .map(|p| self.lower_left + p.to_vector().cast_unit())
                .collect()
        }

        /// Returns the bounding box of the rock in [`ChamberRelativeSpace`].
        fn bounding_box(&self) -> Box2D<isize, ChamberRelativeSpace> {
            Box2D::from_origin_and_size(self.lower_left, self.rock_shape.size().cast_unit())
        }

        /// Returns whether or not `other` collides with this rock.
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
                rock_shape: self.rock_shape,
                lower_left: self.lower_left + rhs,
            }
        }
    }

    /// The result of checking a rock in the chamber buffer.
    #[derive(Debug, Clone, Copy)]
    enum CheckRock {
        /// The rock is at an unencumbered location in the chamber.
        Good,
        /// The rock went outside the chamber.
        OutOfBounds,
        /// The rock collides with a rock in the chamber.
        RockCollision,
        /// The rock fell out of the bottom of the chamber.
        FallOutBottom,
    }

    /// The circular chamber buffer.
    #[derive(Clone, Default, Eq)]
    struct ChamberBuffer {
        /// The buffer of fallen rocks currently in the chamber.
        ///
        /// The bottom of the lowest rock is at relative height zero.
        /// The top of the highest rock is at the relative buffer tower height.
        fallen_rocks: CircularBuffer<BUFFER_SIZE, Rock>,
        /// The absolute floor height of the bottom of the chamber buffer.
        floor_height: Length<u64, ChamberAbsoluteSpace>,
        /// The relative height of the current rock tower in the chamber buffer.
        tower_height: Length<isize, ChamberRelativeSpace>,
        /// The last rock shape that fell and settled into the chamber buffer.
        last_rock_shape: RockShape,
        /// The index of the last jet that pushed the last rock that fell.
        ///
        /// This is relative to the cyclic list of [`JetDirection`]s, and is
        /// needed to search for truly period cycles (part two).
        last_jet_direction_idx: usize,
    }
    impl PartialEq for ChamberBuffer {
        fn eq(&self, other: &Self) -> bool {
            self.fallen_rocks == other.fallen_rocks
                && self.last_rock_shape == other.last_rock_shape
                && self.last_jet_direction_idx == other.last_jet_direction_idx
        }
    }
    impl ChamberBuffer {
        /// Returns the absolute height of the tower of rocks currently in the chamber.
        pub fn tower_height(&self) -> Length<u64, ChamberAbsoluteSpace> {
            self.floor_height + self.tower_height.cast_unit().try_cast().unwrap()
        }

        /// Checks a rock to see how it sits in the current chamber.
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
                } else if rock.lower_left.y <= self.tower_height.0
                    && self.fallen_rocks.iter().any(|r| rock.collides(r))
                {
                    CheckRock::RockCollision
                } else {
                    CheckRock::Good
                }
            }
        }

        /// Adds a rock to the chamber at its current location without performing
        /// any verification checks.
        ///
        /// The oldest rock in the buffer is removed, and the absolute chamber
        /// floor and relative height are adjusted accordingly.
        pub fn add_rock(&mut self, rock: Rock, last_jet_direction_idx: usize) {
            self.last_rock_shape = rock.rock_shape;
            self.last_jet_direction_idx = last_jet_direction_idx;
            self.fallen_rocks.push_front(rock);

            // A rock was removed at the end so re-adjust floor and height.
            let mut floor_offset = isize::MAX;
            let mut height = 0;
            for rock in self.fallen_rocks.iter() {
                floor_offset = floor_offset.min(rock.lower_left.y);
                height = height.max(rock.bounding_box().max.y);
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
    impl std::fmt::Debug for ChamberBuffer {
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

    /// The simulation of rocks falling in the chamber, which is a
    /// [`LendingIterator`].
    struct ChamberSimulation<'a> {
        /// The cyclic iterator of hot air jet directions and their indices in
        /// the original list.
        jet_direction_iter: std::iter::Cycle<
            std::iter::Enumerate<std::iter::Copied<std::slice::Iter<'a, JetDirection>>>,
        >,
        /// A cyclic iterator of the rock shapes.
        rock_shape_iter: std::iter::Cycle<RockShapeIter>,
        /// The current state of the chamber buffer.
        chamber_rocks: ChamberBuffer,
    }
    impl<'a> ChamberSimulation<'a> {
        /// Creates a new simulation given the list of jet directions.
        pub fn new(jet_directions: &'a [JetDirection]) -> Self {
            Self {
                jet_direction_iter: jet_directions.iter().copied().enumerate().cycle(),
                rock_shape_iter: RockShape::iter().cycle(),
                chamber_rocks: ChamberBuffer::default(),
            }
        }
    }
    impl LendingIterator for ChamberSimulation<'_> {
        type Item<'a> = &'a ChamberBuffer
        where
            Self: 'a;

        fn next(&mut self) -> Option<Self::Item<'_>> {
            let rock_shape = self.rock_shape_iter.next().unwrap();

            // Spawn in rock
            let mut rock = Rock::new(
                rock_shape,
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

    /// The chamber, which can parsed from text input.
    #[derive(Debug)]
    pub struct Chamber {
        /// The list of jet directions read from the input.
        jet_directions: Vec<JetDirection>,
    }
    impl FromStr for Chamber {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let jet_directions = s
                .trim()
                .chars()
                .map(JetDirection::try_from)
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
        /// Simulates rocks falling in the tower and returns the overall
        /// rock tower height after `num_rocks` have fallen.
        ///
        /// If `num_rocks` is sufficiently large, a cycle in the chamber buffer is identified
        /// and used to determine the tower height without having to directly simulate `num_rocks`.
        /// This is an optimization necessary to compute part two in a reasonable amount of time.
        pub fn tower_height(&self, num_rocks: usize) -> u64 {
            let mut simulation = ChamberSimulation::new(&self.jet_directions);
            let lcm = lcm(RockShape::COUNT, self.jet_directions.len());

            if num_rocks > lcm {
                // In this case we look for cycles to apply a remainder to reduce the compute time
                let base_chamber_state = simulation.iterations(lcm).unwrap().clone();
                let base_height = base_chamber_state.tower_height();

                // We need to store the heights relative to the base height at each step in the cycle
                let mut relative_heights = Vec::new();
                relative_heights.push(Length::new(0));

                // NOTE: Due to some strange limitation of the borrow checker we cannot  simply
                // use for_each as follows:
                //simulation.take(2 * lcm).for_each(|_| println!("here"));
                let cycle_len = loop {
                    if let Some(cr) = simulation.next() {
                        relative_heights.push(cr.tower_height() - base_height);
                        if *cr == base_chamber_state {
                            break relative_heights.len() - 1;
                        }
                    }
                };

                let (num_cycles, rem) = (num_rocks - lcm).div_rem(&cycle_len);

                base_height.0
                    + relative_heights.last().unwrap().0 * u64::try_from(num_cycles).unwrap()
                    + relative_heights[rem].0
            } else {
                // Directly simulate
                simulation.iterations(num_rocks).unwrap().tower_height().0
            }
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
        |input| {
            // Process
            Ok(input
                .expect_data::<Chamber>()?
                .tower_height(1000000000000)
                .into())
        },
    ],
};
