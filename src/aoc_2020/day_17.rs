use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(386), Unsigned(2276)],
    ".#.
..#
###",
    vec![112u64, 848].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::grid::StdBool;
    use cgmath::Point2;
    use derive_new::new;
    use itertools::Itertools;
    use std::{collections::HashSet, convert::TryInto, fmt::Debug, ops::RangeInclusive};

    /// A range of coordinates containing active cubes for a single dimension.
    type DimensionRange = RangeInclusive<isize>;

    /// A 2D slice of a higher dimensional grid, which can be parsed from text input.
    #[derive(new)]
    pub struct Slice {
        /// Grid for the 2D slice.
        grid: Grid<StdBool>,
    }
    impl From<Grid<StdBool>> for Slice {
        fn from(value: Grid<StdBool>) -> Self {
            Self { grid: value }
        }
    }
    impl Debug for Slice {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Debug::fmt(&self.grid, f)
        }
    }
    impl Slice {
        /// Initialize a new pocket dimension with this slice.
        pub fn initialize_pocket_dimension(&self, dimensions: usize) -> AocResult<PocketDimension> {
            PocketDimension::new(dimensions, self)
        }
    }

    /// An infinite pocket dimension containing Conway cubes in an arbitrary number of dimensions.
    #[derive(Clone)]
    pub struct PocketDimension {
        /// Number of dimensions, e.g. 3 for 3D.
        dimensions: usize,
        /// Set of coordinates of all active Conway cubes.
        active_cubes: HashSet<Vec<isize>>,
    }
    impl Debug for PocketDimension {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let ranges = self.ranges();

            if self.dimensions > 2 {
                for coords in (2..self.dimensions)
                    .map(|i| ranges[i].clone())
                    .multi_cartesian_product()
                {
                    let slice = Slice::new(Grid::<StdBool>::from_coordinates(
                        self.active_cubes
                            .iter()
                            .filter(|pt| pt[2..] == coords)
                            .map(|v| Point2::new(v[0], v[1])),
                    ));

                    writeln!(
                        f,
                        "{}",
                        coords
                            .iter()
                            .enumerate()
                            .map(|(i, v)| format!("x{} = {}", i + 3, v))
                            .join(", ")
                    )?;
                    Debug::fmt(&slice, f)?;
                }
            } else {
                let slice = Slice::new(Grid::<StdBool>::from_coordinates(
                    self.active_cubes.iter().map(|v| Point2::new(v[0], v[1])),
                ));
                Debug::fmt(&slice, f)?;
            }

            Ok(())
        }
    }
    impl PocketDimension {
        /// Create a new pocket dimension from an initial 2D slice.
        fn new(dimensions: usize, initial_slice: &Slice) -> AocResult<Self> {
            if dimensions < 2 {
                return Err(AocError::InvalidInput(
                    format!("Dimension must be at least 2, got {dimensions}").into(),
                ));
            }
            Ok(PocketDimension {
                dimensions,
                active_cubes: initial_slice
                    .grid
                    .as_coordinates()
                    .iter()
                    .map(|p| {
                        let mut v = vec![p.x.try_into().unwrap(), p.y.try_into().unwrap()];
                        v.append(&mut vec![0; dimensions - 2]);
                        v
                    })
                    .collect(),
            })
        }

        /// Verifies that a point has the correct number of dimensions and simply
        /// panics if not.
        fn verify_point(&self, point: &[isize]) {
            if point.len() != self.dimensions {
                panic!(
                "Trying to access a {}-dimensional pocket dimension with a {}-dimensional point",
                self.dimensions,
                point.len()
            )
            }
        }

        /// Returns the inclusive ranges in each dimension that contain active cubes.
        fn ranges(&self) -> Vec<DimensionRange> {
            (0..self.dimensions)
                .map(|i| match self.active_cubes.iter().map(|p| p[i]).range() {
                    Some(r) => r,
                    None => 0..=0,
                })
                .collect()
        }

        /// Counts the number of active cubes.
        pub fn count_active(&self) -> u64 {
            self.active_cubes.len().try_into().unwrap()
        }
    }
    impl Evolver<bool> for PocketDimension {
        type Point = Vec<isize>;

        fn next_default(other: &Self) -> Self {
            PocketDimension {
                dimensions: other.dimensions,
                active_cubes: HashSet::new(),
            }
        }

        fn set_element(&mut self, point: &Self::Point, value: bool) {
            self.verify_point(point);
            if value {
                self.active_cubes.insert(point.clone());
            } else {
                self.active_cubes.remove(point);
            }
        }

        fn next_cell(&self, point: &Self::Point) -> bool {
            self.verify_point(point);
            let neighbors: usize = (0..self.dimensions)
                .map(|i| {
                    let v = point[i];
                    (v - 1)..=(v + 1)
                })
                .multi_cartesian_product()
                .filter_count(|pt| pt != point && self.active_cubes.contains(pt));

            (self.active_cubes.contains(point) && neighbors == 2) || neighbors == 3
        }

        fn next_iter(&self) -> Box<dyn Iterator<Item = Self::Point>> {
            Box::new(
                self.ranges()
                    .iter()
                    .map(|r| (r.start() - 1)..=(r.end() + 1))
                    .multi_cartesian_product(),
            )
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 17,
    name: "Conway Cubes",
    preprocessor: Some(|input| Ok(Box::new(Slice::from_grid_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Slice>()?
                .initialize_pocket_dimension(3)?
                .evolutions()
                .nth(5)
                .unwrap()
                .count_active()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Slice>()?
                .initialize_pocket_dimension(4)?
                .evolutions()
                .nth(5)
                .unwrap()
                .count_active()
                .into())
        },
    ],
};
