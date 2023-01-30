use crate::aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
        vec![Unsigned(1644), Unsigned(229)],
    "11111
19991
19191
19991
11111",
        vec![259u64, 6].answer_vec(),
        "5483143223
2745854711
5264556173
6141336146
6357385478
4167524645
2176841721
6882881134
4846848554
5283751526",
        vec![1656u64, 195].answer_vec()
        }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use std::collections::HashSet;

    /// A grid of octopi, which can be parsed from text input.
    #[derive(Clone)]
    pub struct Octopi {
        /// The grid of octopi energies.
        grid: Grid<u8>,
    }
    impl FromStr for Octopi {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                grid: Grid::from_str::<Grid<u8>>(s)?,
            })
        }
    }
    impl Octopi {
        /// Total number of octopi in the grid.
        pub fn total_octopi(&self) -> usize {
            let size = self.grid.size();
            size.x * size.y
        }

        /// Creates an [Iterator] over the evolution of octopi energies.
        pub fn evolve(self) -> OctopiEvolver {
            OctopiEvolver { octopi: self }
        }
    }

    /// [Iterator] over the evolution of octopi energies, which yields
    /// the number of octopi that flashed at each step.
    pub struct OctopiEvolver {
        /// The octopi grid that evolves.
        octopi: Octopi,
    }
    impl Iterator for OctopiEvolver {
        type Item = u64;

        fn next(&mut self) -> Option<Self::Item> {
            // Fist pass to increment all energies
            for point in self.octopi.grid.all_points() {
                *self.octopi.grid.element_at(&point) += 1;
            }

            // Now repeated passes to look for flashes
            let mut flashes: HashSet<GridPoint> = HashSet::new();
            loop {
                let mut had_flashes = false;

                for point in self.octopi.grid.all_points() {
                    let energy = self.octopi.grid.get(&point);
                    if *energy > 9 && !flashes.contains(&point) {
                        // We have a new flash, increment neighbors
                        let fps: Vec<GridPoint> = self
                            .octopi
                            .grid
                            .neighbor_points(&point, true, false)
                            .collect();
                        for fp in fps {
                            *self.octopi.grid.element_at(&fp) += 1;
                        }

                        // Add flash
                        flashes.insert(point);
                        had_flashes = true;
                    }
                }

                if !had_flashes {
                    break;
                }
            }

            // Lastly, reset all energies that flashed
            for point in flashes.iter() {
                *self.octopi.grid.element_at(point) = 0;
            }

            Some(flashes.len().try_into().unwrap())
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 11,
    name: "Dumbo Octopus",
    preprocessor: Some(|input| Ok(Box::new(Octopi::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Octopi>()?
                .clone()
                .evolve()
                .take(100)
                .sum::<u64>()
                .into())
        },
        // Part two
        |input| {
            // Process
            let octopi = input.expect_data::<Octopi>()?;
            let total_octopi = u64::try_from(octopi.total_octopi()).unwrap();
            Ok((u64::try_from(
                octopi
                    .clone()
                    .evolve()
                    .take_while(|n| *n != total_octopi)
                    .count(),
            )
            .unwrap()
                + 1)
            .into())
        },
    ],
};
