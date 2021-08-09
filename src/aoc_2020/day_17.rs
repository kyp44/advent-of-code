use crate::aoc::prelude::*;
use itertools::Itertools;
use std::{collections::HashSet, convert::TryInto, fmt::Debug, ops::RangeInclusive};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Number;

    solution_test! {
    vec![Number(386), Number(2276)],
    ".#.
..#
###",
    vec![112, 848].answer_vec()
    }
}

type DimensionRange = RangeInclusive<i32>;
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
                            if self.active(&point) {
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

    fn ranges(&self) -> Vec<DimensionRange> {
        (0..self.dimensions)
            .map(|i| {
                let values: Vec<i32> = self.active_cubes.iter().map(|p| p[i]).collect();
                (*values.iter().min().unwrap_or(&0))..=(*values.iter().max().unwrap_or(&0))
            })
            .collect()
    }

    fn active(&self, point: &[i32]) -> bool {
        self.active_cubes.contains(point)
    }

    fn count_active(&self) -> usize {
        self.active_cubes.len()
    }

    fn count_neighbors_active(&self, point: &[i32]) -> usize {
        fn point_range(val: &i32) -> DimensionRange {
            (val - 1)..=(val + 1)
        }
        (0..self.dimensions)
            .map(|i| point_range(&point[i]))
            .multi_cartesian_product()
            .filter_count(|pt| pt != point && self.active(pt))
    }

    fn next(&self) -> Dimension {
        fn exp_range(range: &DimensionRange) -> DimensionRange {
            (range.start() - 1)..=(range.end() + 1)
        }
        let ranges = self.ranges();

        Dimension {
            dimensions: self.dimensions,
            active_cubes: (0..self.dimensions)
                .map(|i| exp_range(&ranges[i]))
                .multi_cartesian_product()
                .filter(|point| {
                    let num = self.count_neighbors_active(point);

                    (self.active(point) && num == 2) || num == 3
                })
                .collect(),
        }
    }

    fn run(&self, cycles: usize) -> Dimension {
        let mut current = self.next();
        for _ in 1..cycles {
            //println!("After {} cycles:\n{:?}", i, current);
            current = current.next();
        }
        current
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

            // Process
            Ok(Answer::Number(
                dimension.run(6).count_active().try_into().unwrap(),
            ))
        },
        // Part b)
        |input| {
            // Generation
            let dimension = Dimension::from_str(4, input)?;

            // Process
            Ok(Answer::Number(
                dimension.run(6).count_active().try_into().unwrap(),
            ))
        },
    ],
};
