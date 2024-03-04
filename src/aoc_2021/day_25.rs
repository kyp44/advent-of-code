use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
    example {
        input = "v...>>.vv>
.vv>>.vv..
>>.>v>...v
>>v>>.>.v.
v>v.vv.v..
>.>>..v...
.vv..>.>v.
v.v..>>v.v
....v..v.>";
        answers = unsigned![58];
    }
    actual_answers = unsigned![523];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::grid::GridSpace;
    use euclid::Vector2D;
    use std::{fmt, rc::Rc};

    /// A spot in the [`Trench`] grid.
    #[derive(PartialEq, Eq, Clone, Copy, Default)]
    pub enum Location {
        /// Empty space.
        #[default]
        Empty,
        /// Cucumber that is moving East.
        East,
        /// Cucumber that is moving South.
        South,
    }
    impl From<&Location> for char {
        fn from(value: &Location) -> Self {
            match value {
                Location::Empty => '.',
                Location::East => '>',
                Location::South => 'v',
            }
        }
    }
    impl fmt::Debug for Location {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", char::from(self))
        }
    }
    impl TryFrom<char> for Location {
        type Error = ();

        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                '.' => Ok(Self::Empty),
                '>' => Ok(Self::East),
                'v' => Ok(Self::South),
                _ => Err(()),
            }
        }
    }
    impl Location {
        /// Returns whether or not this location is occupied.
        fn occupied(&self) -> bool {
            !matches!(self, Location::Empty)
        }
    }

    /// The state of a trench, which is a [`Grid`] of [`Location`]s.
    #[derive(Clone, PartialEq, Eq)]
    pub struct Trench {
        /// Grid of trench locations.
        grid: Grid<Location>,
    }
    impl From<Grid<Location>> for Trench {
        fn from(value: Grid<Location>) -> Self {
            Trench { grid: value }
        }
    }
    impl fmt::Debug for Trench {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "{:?}", self.grid)
        }
    }
    impl Trench {
        /// Returns an [`Iterator`] over the points in the trench that contain
        /// a particular type of [`Location`].
        fn specific_points<'a>(
            &'a self,
            location: &'a Location,
        ) -> impl Iterator<Item = AnyGridPoint> + 'a {
            self.grid.all_points().filter_map(move |point| {
                if self.grid.get(&point) == location {
                    Some(point.to_isize())
                } else {
                    None
                }
            })
        }

        /// Determines and returns the next trench state.
        fn next(&self) -> Self {
            let mut new_trench = self.clone();

            // Move all cucumbers of a particular type
            let mut move_cucumbers = |cucumber: Location, direction: Vector2D<isize, GridSpace>| {
                // Need to capture the current state to check against for all open spaces.
                // This fixes the problem of a cucumber in the first row/col moving out of the way first
                // so that one in the last row/col moves in, which shouldn't happen.
                let check_trench = new_trench.clone();

                for point in self.specific_points(&cucumber) {
                    let moved_point =
                        (point + direction).wrapped_grid_point(new_trench.grid.size());
                    // Move if the adjacent space is free
                    if !check_trench.grid.get(&moved_point).occupied() {
                        new_trench.grid.set(
                            &point.wrapped_grid_point(new_trench.grid.size()),
                            Location::Empty,
                        );
                        new_trench.grid.set(&moved_point, cucumber);
                    }
                }
            };

            // Move all eastbound cucumbers
            move_cucumbers(Location::East, Vector2D::new(1, 0));

            // Move all southbound cucumbers
            move_cucumbers(Location::South, Vector2D::new(0, 1));

            new_trench
        }

        /// Returns an [`Iterator`] over the tranche states.
        ///
        /// The first element of the [`Iterator`] will be the next state after this one.
        pub fn iter(&self) -> TrenchIter {
            TrenchIter {
                current: Rc::new(self.clone()),
            }
        }

        /// Returns the first step number for which no cucumbers were able to move.
        pub fn first_step_with_no_movement(&self) -> usize {
            self.iter().count() + 1
        }
    }

    /// Iterator over the evolution of trench states.
    ///
    /// The iterator terminates after the last step during which cucumbers were able to move.
    /// That is, [`None`] is returned from [`TrenchIter::next`] if no cucumbers can move.
    pub struct TrenchIter {
        /// The current [`Trench`], which is the one the iterator previously returned.
        current: Rc<Trench>,
    }
    impl Iterator for TrenchIter {
        type Item = Rc<Trench>;

        fn next(&mut self) -> Option<Self::Item> {
            let next = Rc::new(self.current.as_ref().next());
            if next == self.current {
                None
            } else {
                self.current = next;
                Some(self.current.clone())
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 25,
    name: "Sea Cucumber",
    preprocessor: Some(|input| Ok(Box::new(Trench::from_grid_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            let steps: u64 = input
                .expect_data::<Trench>()?
                .first_step_with_no_movement()
                .try_into()
                .unwrap();
            Ok(steps.into())
        },
    ],
};
