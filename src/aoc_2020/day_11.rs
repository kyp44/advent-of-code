use std::{cell::RefCell, str::FromStr};

use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(2483), Unsigned(2285)],
    "L.LL.LL.LL
LLLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLLL
L.LLLLLL.L
L.LLLLL.LL",
        vec![37u64, 26].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use cgmath::{Vector2, Zero};
    use derive_new::new;
    use itertools::iproduct;
    use std::{collections::HashSet, fmt::Display, hash::Hash, rc::Rc};

    /// State of a single seat in the waiting room.
    #[derive(Clone, Copy, Hash, PartialEq, Eq, Default)]
    pub enum Seat {
        /// No seat here, just the floor.
        #[default]
        Floor,
        /// The seat is empty.
        Empty,
        /// The seat is occupied.
        Occupied,
    }
    impl From<char> for Seat {
        fn from(c: char) -> Self {
            match c {
                'L' => Seat::Empty,
                '#' => Seat::Occupied,
                _ => Seat::Floor,
            }
        }
    }
    impl From<&Seat> for char {
        fn from(s: &Seat) -> Self {
            match *s {
                Seat::Floor => '.',
                Seat::Empty => 'L',
                Seat::Occupied => '#',
            }
        }
    }

    /// The termination status of a waiting room simulation.
    pub enum TerminationStatus<T> {
        /// The simulation becomes stable (i.e. unchanging) with the final state.
        Stable(T),
        /// The simulation evolves forever, becoming periodic at some point with the first repeated state.
        Periodic(T),
    }
    impl<T: Display> Display for TerminationStatus<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let (s, a) = match self {
                TerminationStatus::Stable(a) => ("Stable", a),
                TerminationStatus::Periodic(a) => ("Infinite", a),
            };
            write!(f, "{s}:\n{a}")
        }
    }
    impl TerminationStatus<Rc<Area>> {
        /// The the simulation result to ensure that its stable.
        ///
        /// The `Ok` return variant contains the number of occupied seats.
        pub fn check(&self) -> AocResult<u64> {
            match self {
                TerminationStatus::Stable(a) => Ok(a.count_occupied()),
                TerminationStatus::Periodic(_) => Err(AocError::Process(
                    "Simulation did not reach a steady state".into(),
                )),
            }
        }
    }

    /// Part of the problem.
    ///
    /// This must be an enum because the behavior of the [`Evolver`] methods depend
    /// on the part so it cannot be a generic parameter (since the trait method
    /// has no generic parameters).
    #[derive(Clone, Copy, Hash, PartialEq, Eq)]
    pub enum Part {
        /// Part one.
        PartOne,
        /// Part two.
        PartTwo,
    }
    impl Part {
        /// Minimum number of adjacent seats that are occupied in order for a seat to vacate.
        fn min_needed_to_vacate(&self) -> u8 {
            match self {
                Part::PartOne => 4,
                Part::PartTwo => 5,
            }
        }

        /// Number of occupied seats for a given seat for the part.
        fn point_occupied(&self, area: &Area, point: &GridPoint) -> u8 {
            let grid = &area.grid;
            match self {
                // Just look at the eight adjacent seats.
                Part::PartOne => grid
                    .neighbor_points(point, true, false)
                    .filter_count(|point| *grid.get(point) == Seat::Occupied),
                // Look for the first seat in the eight directions in our line of sight.
                Part::PartTwo => iproduct!(-1isize..=1, -1isize..=1)
                    .map(|(dx, dy)| Vector2::new(dx, dy))
                    .filter(|dp| *dp != Vector2::zero())
                    .filter_count(|dp| {
                        let mut i = 1;
                        loop {
                            let point = point.try_point_into().unwrap();

                            match grid.valid_point(&(point + i * dp)) {
                                Some(p) => match grid.get(&p) {
                                    Seat::Occupied => break true,
                                    Seat::Empty => break false,
                                    Seat::Floor => (),
                                },
                                None => break false,
                            }

                            i += 1;
                        }
                    }),
            }
        }
    }

    /// The waiting room seating area, which can be parsed from text input.
    #[derive(Clone, Hash, PartialEq, Eq, new)]
    pub struct Area {
        /// The part for whose rules we want to simulate.
        part: Part,
        /// The grid of seats.
        grid: Grid<Seat>,
    }
    impl CharGrid<Seat> for Area {
        fn get_grid(&self) -> &Grid<Seat> {
            &self.grid
        }

        fn from_char(c: char) -> Option<Seat> {
            Some(c.into())
        }

        fn to_char(e: &Seat) -> char {
            e.into()
        }
    }
    impl FromStr for Area {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Area {
                part: Part::PartOne,
                grid: Grid::from_str::<Self>(s)?,
            })
        }
    }
    impl Evolver<Seat> for Area {
        type Point = GridPoint;

        fn next_default(other: &Self) -> Self {
            Area::new(other.part, Grid::default(*other.grid.size()))
        }

        fn get_element(&self, point: &Self::Point) -> Seat {
            *self.grid.get(point)
        }

        fn set_element(&mut self, point: &Self::Point, value: Seat) {
            self.grid.set(point, value)
        }

        fn next_cell(&self, point: &Self::Point) -> Seat {
            let occupied = self.part.point_occupied(self, point);
            let orig = self.get_element(point);
            match orig {
                Seat::Empty => {
                    if occupied == 0 {
                        Seat::Occupied
                    } else {
                        orig
                    }
                }
                Seat::Occupied => {
                    if occupied >= self.part.min_needed_to_vacate() {
                        Seat::Empty
                    } else {
                        orig
                    }
                }
                _ => orig,
            }
        }

        fn next_iter(&self) -> Box<dyn Iterator<Item = Self::Point>> {
            Box::new(self.grid.all_points())
        }
    }
    impl Area {
        /// Set the problem at to use.
        pub fn set_part(&mut self, part: Part) {
            self.part = part;
        }

        /// Counts the number of occupied seats.
        fn count_occupied(&self) -> u64 {
            self.grid
                .all_values()
                .filter_count(|seat| matches!(seat, Seat::Occupied))
        }

        /// Run the simulation and returns the termination status containing the final state.
        pub fn simulate(&self) -> TerminationStatus<Rc<Self>> {
            let mut prior_states: HashSet<Rc<Self>> = HashSet::new();
            let mut last_state = prior_states.get_or_insert(Rc::new(self.clone()));
            for state in self.evolutions() {
                //println!("{:?}", state);
                if state == *last_state {
                    return TerminationStatus::Stable(state);
                }
                if prior_states.contains(&state) {
                    return TerminationStatus::Periodic(state);
                }
                last_state = prior_states.get_or_insert(state);
            }
            panic!("Somehow the evolver iterator ended!")
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 11,
    name: "Seating System",
    preprocessor: Some(|input| Ok(Box::new(RefCell::new(Area::from_str(input)?)).into())),
    solvers: &[
        // Part one
        |input| {
            // Generation
            let area = input.expect_data::<RefCell<Area>>()?;
            area.borrow_mut().set_part(Part::PartOne);

            // Process
            Ok(area.borrow().simulate().check()?.into())
        },
        // Part two
        |input| {
            // Generation
            let area = input.expect_data::<RefCell<Area>>()?;
            area.borrow_mut().set_part(Part::PartTwo);

            // Process
            Ok(area.borrow().simulate().check()?.into())
        },
    ],
};
