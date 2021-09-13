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

#[derive(CharGridDebug)]
struct Slice2D {
    data: Box<[Box<[bool]>]>,
}
impl CharGrid for Slice2D {
    type Element = bool;

    fn default() -> Self::Element {
        false
    }

    fn from_char(c: char) -> Self::Element {
        c == '#'
    }

    fn to_char(e: &Self::Element) -> char {
        if *e {
            '#'
        } else {
            '.'
        }
    }

    fn from_data(_size: (usize, usize), data: Box<[Box<[Self::Element]>]>) -> AocResult<Self>
    where
        Self: Sized,
    {
        Ok(Slice2D { data })
    }

    fn to_data(&self) -> &[Box<[Self::Element]>] {
        &self.data
    }
}

#[derive(Clone)]
struct Dimension {
    dimensions: usize,
    active_cubes: HashSet<Vec<i32>>,
}
impl Debug for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ranges = self.ranges();

        if self.dimensions > 2 {
            for coords in (2..self.dimensions)
                .map(|i| ranges[i].clone())
                .multi_cartesian_product()
            {
                let slice = Slice2D::from_coordinates(
                    &self
                        .active_cubes
                        .iter()
                        .filter(|pt| pt[2..] == coords)
                        .map(|v| (v[0], v[1]))
                        .collect(),
                )
                .unwrap();
                writeln!(
                    f,
                    "{}",
                    coords
                        .iter()
                        .enumerate()
                        .map(|(i, v)| format!("x{} = {}", i + 3, v))
                        .join(", ")
                )?;
                slice.fmt(f)?;
            }
        } else {
            let slice = Slice2D::from_coordinates(
                &self.active_cubes.iter().map(|v| (v[0], v[1])).collect(),
            )
            .unwrap();
            slice.fmt(f)?;
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
            active_cubes: Slice2D::from_str(s)?
                .to_coordinates()
                .iter()
                .map(|(x, y)| {
                    let mut v = vec![*x, *y];
                    v.append(&mut vec![0; dimensions - 2]);
                    v
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
            .map(|i| match self.active_cubes.iter().map(|p| p[i]).range() {
                Some(r) => r,
                None => 0..=0,
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
            for dim in dimension.evolutions().take(5) {
                println!("{:?}", dim);
            }*/

            // Process
            Ok(dimension.evolutions().nth(5).unwrap().count_active().into())
        },
        // Part b)
        |input| {
            // Generation
            let dimension = Dimension::from_str(4, input)?;

            /*println!("{:?}", dimension);
            for dim in dimension.evolutions().take(5) {
                println!("{:?}", dim);
            }*/

            // Process
            Ok(dimension.evolutions().nth(5).unwrap().count_active().into())
        },
    ],
};
