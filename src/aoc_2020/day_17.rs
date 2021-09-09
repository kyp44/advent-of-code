use crate::aoc::prelude::*;
use itertools::Itertools;
use std::{collections::HashSet, convert::TryInto, fmt::Debug, ops::RangeInclusive};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(386), Unsigned(2276)],
    ".#.
..#
###",
    vec![112u64, 848].answer_vec()
    }
}

type DimensionRange = RangeInclusive<i32>;

#[derive(Clone)]
struct Dimension {
    dimensions: usize,
    active_cubes: HashSet<Vec<i32>>,
}
impl Debug for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ranges = self.ranges();

        let print_grid = |f: &mut std::fmt::Formatter<'_>, cs: Vec<i32>| -> std::fmt::Result {
            for y in ranges[1].clone() {
                writeln!(
                    f,
                    "{}",
                    ranges[0]
                        .clone()
                        .map(|x| {
                            let mut point = vec![x, y];
                            point.extend_from_slice(&cs);
                            if self.get(&point) {
                                '#'
                            } else {
                                '.'
                            }
                        })
                        .collect::<String>()
                )?;
            }
            Ok(())
        };

        if self.dimensions > 2 {
            for coords in (2..self.dimensions)
                .map(|i| ranges[i].clone())
                .multi_cartesian_product()
            {
                writeln!(
                    f,
                    "{}",
                    coords
                        .iter()
                        .enumerate()
                        .map(|(i, v)| format!("x{} = {}", i + 3, v))
                        .join(", ")
                )?;
                print_grid(f, coords)?;
            }
        } else {
            print_grid(f, vec![])?;
        }

        Ok(())
    }
}
impl Dimension {
    fn from_str(dimensions: usize, s: &str) -> AocResult<Self> {
        if dimensions < 2 {
            return Err(AocError::InvalidInput(
                format!("Dimension must be at least 2, got {}", dimensions).into(),
            ));
        }
        Ok(Dimension {
            dimensions,
            active_cubes: s
                .lines()
                .enumerate()
                .flat_map(|(y, line)| {
                    line.chars().enumerate().filter_map(move |(x, c)| match c {
                        '.' => None,
                        _ => {
                            let mut point = vec![x.try_into().unwrap(), y.try_into().unwrap()];
                            point.resize(dimensions, 0);
                            Some(point)
                        }
                    })
                })
                .collect(),
        })
    }

    fn verify_point(&self, point: &[i32]) {
        if point.len() != self.dimensions {
            panic!(
                "Trying to access a {}-dimensional pocket dimension with a {}-dimensional point",
                self.dimensions,
                point.len()
            )
        }
    }

    fn ranges(&self) -> Vec<DimensionRange> {
        (0..self.dimensions)
            .map(|i| {
                let values: Vec<i32> = self.active_cubes.iter().map(|p| p[i]).collect();
                (*values.iter().min().unwrap_or(&0))..=(*values.iter().max().unwrap_or(&0))
            })
            .collect()
    }

    fn count_active(&self) -> u64 {
        self.active_cubes.len().try_into().unwrap()
    }
}

impl Evolver<bool> for Dimension {
    type Point = Vec<i32>;
    type Iter = impl Iterator<Item = Self::Point>;

    fn new(other: &Self) -> Self {
        Dimension {
            dimensions: other.dimensions,
            active_cubes: HashSet::new(),
        }
    }

    fn get(&self, point: &Self::Point) -> bool {
        self.verify_point(point);
        self.active_cubes.contains(point)
    }

    fn set(&mut self, point: &Self::Point, value: bool) {
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
            .filter_count(|pt| pt != point && self.get(pt));

        (self.get(point) && neighbors == 2) || neighbors == 3
    }

    fn next_iter(&self) -> Self::Iter {
        self.ranges()
            .iter()
            .map(|r| (r.start() - 1)..=(r.end() + 1))
            .into_iter()
            .multi_cartesian_product()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 17,
    name: "Conway Cubes",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let dimension = Dimension::from_str(3, input)?;

            /*println!("{:?}", dimension);
            for dim in dimension.evolutions().take(3) {
                println!("{:?}", dim);
            }*/

            // Process
            Ok(dimension.evolutions().nth(5).unwrap().count_active().into())
        },
        // Part b)
        |input| {
            // Generation
            let dimension = Dimension::from_str(4, input)?;

            // Process
            Ok(dimension.evolutions().nth(5).unwrap().count_active().into())
        },
    ],
};
