use std::{collections::HashSet, str::FromStr};

use crate::aoc::prelude::*;

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

#[derive(Clone)]
struct Octopi {
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
    fn evolve(self) -> OctopiEvolver {
        OctopiEvolver { octopi: self }
    }

    fn energy(&mut self, point: &GridPoint) -> &mut u8 {
        self.grid.element_at(point)
    }

    fn energy_iter(&self) -> impl Iterator<Item = GridPoint> {
        self.grid.all_points()
    }
}

struct OctopiEvolver {
    octopi: Octopi,
}
impl Iterator for OctopiEvolver {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        // Fist pass to increment all energies
        for point in self.octopi.energy_iter() {
            *self.octopi.energy(&point) += 1;
        }

        // Now repeated passes to look for flashes
        let mut flashes: HashSet<GridPoint> = HashSet::new();
        loop {
            let mut had_flashes = false;

            for point in self.octopi.energy_iter() {
                let energy = self.octopi.energy(&point);
                if *energy > 9 && !flashes.contains(&point) {
                    // We have a new flash, increment neighbors
                    let fps: Vec<GridPoint> = self
                        .octopi
                        .grid
                        .neighbor_points(&point, true, false)
                        .collect();
                    for fp in fps {
                        *self.octopi.energy(&fp) += 1;
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
            *self.octopi.energy(point) = 0;
        }

        Some(flashes.len().try_into().unwrap())
    }
}

pub const SOLUTION: Solution = Solution {
    day: 11,
    name: "Dumbo Octopus",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let octopi = Octopi::from_str(input.expect_input()?)?;

            /*let mut evolver = octopi.clone().evolve();
            for _ in 0..5 {
                println!("{:?}", evolver.octopi);
                println!("Flashes: {}\n", evolver.next().unwrap());
            }*/

            // Process
            Ok(octopi.evolve().take(100).sum::<u64>().into())
        },
        // Part two
        |input| {
            // Generation
            let octopi = Octopi::from_str(input.expect_input()?)?;
            let size = octopi.grid.size();
            let total_octopi = u64::try_from(size.x * size.y).unwrap();

            // Process
            Ok(
                (u64::try_from(octopi.evolve().take_while(|n| *n != total_octopi).count())
                    .unwrap()
                    + 1)
                .into(),
            )
        },
    ],
};
