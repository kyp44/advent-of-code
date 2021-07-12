use std::{collections::HashSet, convert::TryInto, fmt::Debug, ops::RangeInclusive, str::FromStr};

use itertools::iproduct;

use crate::aoc::{AocError, FilterCount, Solution};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![386],
    ".#.
..#
###",
    vec![Some(112)]
    }
}

const DIM: usize = 3;

type DimensionRange = RangeInclusive<i32>;
struct Dimension {
    active_cubes: HashSet<[i32; DIM]>,
}
impl FromStr for Dimension {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Dimension {
            active_cubes: s
                .lines()
                .enumerate()
                .flat_map(|(y, line)| {
                    line.chars().enumerate().filter_map(move |(x, c)| match c {
                        '.' => None,
                        _ => Some([x.try_into().unwrap(), y.try_into().unwrap(), 0]),
                    })
                })
                .collect(),
        })
    }
}
impl Debug for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ranges = self.ranges();
        for z in ranges[2].clone() {
            writeln!(f, "\nz = {}", z)?;
            for y in ranges[1].clone() {
                writeln!(
                    f,
                    "{}",
                    ranges[0]
                        .clone()
                        .map(|x| if self.active(&[x, y, z]) { '#' } else { '.' })
                        .collect::<String>()
                )?;
            }
        }

        Ok(())
    }
}
impl Dimension {
    fn ranges(&self) -> [DimensionRange; DIM] {
        let ranges: Vec<DimensionRange> = (0..DIM)
            .into_iter()
            .map(|i| {
                let values: Vec<i32> = self.active_cubes.iter().map(|p| p[i]).collect();
                (*values.iter().min().unwrap_or(&0))..=(*values.iter().max().unwrap_or(&0))
            })
            .collect();
        ranges.try_into().unwrap()
    }

    fn active(&self, point: &[i32]) -> bool {
        self.active_cubes.contains(point)
    }

    fn count_active(&self) -> usize {
        self.active_cubes.len()
    }

    fn count_neighbors_active(&self, point: &[i32]) -> usize {
        fn ran(val: &i32) -> DimensionRange {
            (val - 1)..=(val + 1)
        }
        iproduct!(ran(&point[0]), ran(&point[1]), ran(&point[2]))
            .filter_count(|(x, y, z)| [*x, *y, *z] != point && self.active(&[*x, *y, *z]))
    }

    fn next(&self) -> Dimension {
        fn exp(range: &DimensionRange) -> DimensionRange {
            (range.start() - 1)..=(range.end() + 1)
        }
        let ranges = self.ranges();

        Dimension {
            active_cubes: iproduct!(exp(&ranges[0]), exp(&ranges[1]), exp(&ranges[2]))
                .filter_map(|(x, y, z)| {
                    let point = [x, y, z];
                    let num = self.count_neighbors_active(&point);
                    if self.active(&point) {
                        if num == 2 || num == 3 {
                            return Some(point);
                        }
                    } else {
                        if num == 3 {
                            return Some(point);
                        }
                    }
                    None
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
            let dimension = Dimension::from_str(input)?;

            // Process
            Ok(dimension.run(6).count_active().try_into().unwrap())
        },
    ],
};
