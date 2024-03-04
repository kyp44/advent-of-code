use aoc::prelude::*;
use gat_lending_iterator::LendingIterator;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "498,4 -> 498,6 -> 496,6
    503,4 -> 502,4 -> 502,9 -> 494,9";
            answers = unsigned![24, 93];
        }
        actual_answers = unsigned![578, 24377];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::collections::HashSet;

    use super::*;
    use aoc::parse::trim;
    use derive_more::Deref;
    use derive_new::new;
    use euclid::{vec2, Point2D};
    use gat_lending_iterator::LendingIterator;
    use itertools::process_results;
    use nom::{
        bytes::complete::tag, combinator::map, multi::separated_list1, sequence::separated_pair,
    };

    /// A point that can be parsed from the rock line definition text.
    #[derive(Deref)]
    struct ParsePoint(AnyGridPoint);
    impl Parsable<'_> for ParsePoint {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(
                    nom::character::complete::u16,
                    tag(","),
                    nom::character::complete::u16,
                ),
                |(x, y)| ParsePoint(Point2D::new(x, y).to_isize()),
            )(input)
        }
    }

    /// A line of rocks.
    #[derive(new)]
    struct RockLine {
        /// The points that define the horizontal and vertical segments of the line.
        segment_points: Vec<AnyGridPoint>,
    }
    impl Parsable<'_> for RockLine {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                trim(
                    false,
                    separated_list1(trim(false, tag("->")), ParsePoint::parser),
                ),
                |pps| Self {
                    segment_points: pps.into_iter().map(|pp| *pp).collect(),
                },
            )(input)
        }
    }
    impl RockLine {
        /// Returns the set of all the points that form the entire line, or any
        /// error if the line definition is not valid.
        pub fn points(&self) -> AocResult<HashSet<AnyGridPoint>> {
            let points: HashSet<_> = process_results(
                self.segment_points.windows(2).map(
                    |win| -> AocResult<Box<dyn Iterator<Item = AnyGridPoint>>> {
                        let p1 = &win[0];
                        let p2 = &win[1];

                        if p1.x == p2.x {
                            let (y1, y2) = (p1.y.min(p2.y), p1.y.max(p2.y));
                            Ok(Box::new((y1..=y2).map(|y| AnyGridPoint::new(p1.x, y))))
                        } else if p1.y == p2.y {
                            let (x1, x2) = (p1.x.min(p2.x), p1.x.max(p2.x));
                            Ok(Box::new((x1..=x2).map(|x| AnyGridPoint::new(x, p1.y))))
                        } else {
                            Err(AocError::Process(
                                format!(
                                "Points {:?} and {:?} do not form a horizontal or vertical line",
                                p1, p2
                            )
                                .into(),
                            ))
                        }
                    },
                ),
                |iter| iter.flatten().collect(),
            )?;

            if points.is_empty() {
                Err(AocError::Process(
                    "The line has less than two points!".into(),
                ))
            } else {
                Ok(points)
            }
        }
    }

    /// A cell in the grid representation of the cave chamber.
    #[derive(Clone)]
    enum Cell {
        /// The cell is empty, containing only air.
        Air,
        /// The cell contains rock.
        Rock,
        /// The cell contains a grain of sand.
        Sand,
        /// The cell is the source of the sand into the chamber.
        Source,
    }
    impl Default for Cell {
        fn default() -> Self {
            Self::Air
        }
    }
    impl From<bool> for Cell {
        fn from(value: bool) -> Self {
            if value {
                Self::Rock
            } else {
                Self::default()
            }
        }
    }
    impl std::fmt::Debug for Cell {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Self::Air => '.',
                    Self::Rock => '#',
                    Self::Sand => 'o',
                    Self::Source => '+',
                }
            )
        }
    }
    impl Cell {
        /// Returns whether or not the cell is air, otherwise it is occupied.
        pub fn is_air(&self) -> bool {
            matches!(self, Self::Air)
        }
    }

    /// Represents the cave chamber, which can be parsed from text input.
    pub struct Cave {
        /// The collection of lines of rock.
        rock_lines: Vec<RockLine>,
    }
    impl FromStr for Cave {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                rock_lines: RockLine::gather(s.lines())?,
            })
        }
    }
    impl Cave {
        /// Begins the simulation of sand entering the chamber.
        ///
        /// The simulation can be run with or without taking into account the cave
        /// floor (part two) or not (part one).
        pub fn simulate_sand(&self, use_floor: bool) -> AocResult<SimulationState> {
            let mut rock_points = HashSet::new();

            // Add rock line points
            for rock_line in self.rock_lines.iter() {
                rock_points.extend(rock_line.points()?.into_iter());
            }

            // Check that all rocks are below the source
            if rock_points.iter().map(|p| p.y).min().unwrap() <= 0 {
                return Err(AocError::Process(
                    "The are rocks at or above the sand source!".into(),
                ));
            }

            // Add phony rock where the source will be placed.
            // This is needed to ensure that the source row exists in the grid.
            let source = AnyGridPoint::new(500, 0);
            rock_points.insert(source);

            // Add the floor if selected
            if use_floor {
                let fy = rock_points.iter().map(|p| p.y).max().unwrap() + 2;

                // Calculate floor x bounds based on diagonal projection down from every rock and the source.
                let fx1 = rock_points.iter().map(|p| p.x - (fy - p.y)).min().unwrap();
                let fx2 = rock_points.iter().map(|p| p.x + (fy - p.y)).max().unwrap();

                rock_points.extend(
                    RockLine::new(vec![AnyGridPoint::new(fx1, fy), AnyGridPoint::new(fx2, fy)])
                        .points()?,
                )
            }

            // Figure out the displacement in x between the points and the grid coordinate
            let delta = rock_points.iter().map(|p| p.x).min().unwrap();

            // Create the grid
            let mut grid: Grid<Cell> = Grid::from_coordinates(rock_points.iter());

            // Mark the source location in the grid
            let source = source - vec2(1, 0) * delta;
            grid.set_any(&source, Cell::Source);

            Ok(SimulationState { grid, source })
        }
    }

    /// An [`LendingIterator`] over the states of the cave chamber as sand enters.
    ///
    /// Each element is a reference to the state after each additional grain of sand
    /// enters at the source, falls, and comes to rest.
    /// The iterator terminates once either sand is falling down into the void
    /// (part one) or the source is completely blocked with sand because of it
    /// piling up on the floor (part two).
    pub struct SimulationState {
        /// The current grid representation of the cave chamber.
        grid: Grid<Cell>,
        /// The location of the sand source in the `grid`.
        source: AnyGridPoint,
    }
    impl LendingIterator for SimulationState {
        type Item<'a> = &'a SimulationState
        where
            Self: 'a;

        fn next(&mut self) -> Option<Self::Item<'_>> {
            // Create a new grain of sand and let it fall according to the rules
            let mut point = self.source;

            // The source is blocked we are done
            if let Cell::Sand = self.grid.get_any(&point).unwrap() {
                return None;
            }

            let deltas = [vec2(0, 1), vec2(-1, 1), vec2(1, 1)];

            loop {
                let mut deltas = deltas.iter();
                loop {
                    match deltas.next() {
                        Some(delta) => {
                            let new = point + *delta;
                            match self.grid.get_any(&new) {
                                Some(cell) => {
                                    if cell.is_air() {
                                        // We can move the grain here
                                        point = new;
                                        break;
                                    }
                                }
                                None => {
                                    // the grain has gone off the board, so we are done
                                    return None;
                                }
                            }
                        }
                        None => {
                            // The grain cannot move, so come to rest and the step is over
                            self.grid.set_any(&point, Cell::Sand);
                            return Some(self);
                        }
                    }
                }
            }
        }
    }
    impl std::fmt::Debug for SimulationState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.grid)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 14,
    name: "Regolith Reservoir",
    preprocessor: Some(|input| Ok(Box::new(Cave::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<Cave>()?
                    .simulate_sand(false)?
                    .count()
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<Cave>()?
                    .simulate_sand(true)?
                    .count()
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
