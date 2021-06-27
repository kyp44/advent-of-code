use crate::aoc::{AocError, FilterCount, Solution};
use itertools::iproduct;
use std::{cmp::min, collections::HashSet, fmt::Display, hash::Hash, str::FromStr};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![2483],
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
        vec![37]
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Seat {
    Floor,
    Empty,
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

enum SimulationStatus<T> {
    Stable(T),
    Infinite(T),
}

impl<T: Display> Display for SimulationStatus<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (s, a) = match self {
            SimulationStatus::Stable(a) => ("Stable", a),
            SimulationStatus::Infinite(a) => ("Infinite", a),
        };
        write!(f, "{}:\n{}", s, a)
    }
}

/// Marker structs for each part.
/// This really shows the power of abstraction of Rust to accomplish
/// multiple implementations of the same trait.
trait Part {
    fn min_needed_to_vacate() -> u32;
}
struct PartA;
struct PartB;
impl Part for PartA {
    fn min_needed_to_vacate() -> u32 {
        4
    }
}
impl Part for PartB {
    fn min_needed_to_vacate() -> u32 {
        5
    }
}

trait Simulator {
    fn size(&self) -> (usize, usize);
    fn get(&self, point: (usize, usize)) -> Seat;
    fn set(&mut self, point: (usize, usize), val: Seat);
}

trait SimulatorPart<T: Part>
where
    Self: Sized + Clone + Simulator + Hash + Eq,
{
    fn point_occupied(&self, point: (usize, usize)) -> u32;

    fn next(&self) -> Self {
        let mut next = (*self).clone();
        let size = self.size();

        for (x, y) in iproduct!(0..size.0, 0..size.1) {
            let occupied = self.point_occupied((x, y));
            let orig = self.get((x, y));
            //println!("TODO seat ({}, {}) = {}", x, y, occupied);
            next.set(
                (x, y),
                match orig {
                    Seat::Empty => {
                        if occupied == 0 {
                            Seat::Occupied
                        } else {
                            orig
                        }
                    }
                    Seat::Occupied => {
                        if occupied >= T::min_needed_to_vacate() {
                            Seat::Empty
                        } else {
                            orig
                        }
                    }
                    _ => orig,
                },
            );
        }
        //println!("\n{}", next);
        next
    }

    fn simulate(&self) -> SimulationStatus<Self> {
        let mut prior_states: HashSet<Self> = HashSet::new();
        let mut last_state = prior_states.get_or_insert(self.clone());
        loop {
            let next = last_state.next();
            if next == *last_state {
                break SimulationStatus::Stable(next);
            }
            if prior_states.contains(&next) {
                break SimulationStatus::Infinite(next);
            }
            last_state = prior_states.get_or_insert(next);
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct Area {
    width: usize,
    height: usize,
    data: Vec<Vec<Seat>>,
}

impl FromStr for Area {
    type Err = AocError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut liter = s.lines();
        fn parse_row(line: &str) -> Vec<Seat> {
            line.trim().chars().map(|c| c.into()).collect()
        }
        let first_row = parse_row(
            liter
                .next()
                .ok_or_else(|| AocError::InvalidInput("No lines".to_string()))?,
        );
        let width = first_row.len();
        if width < 1 {
            return Err(AocError::InvalidInput(
                "First map line has no content!".to_string(),
            ));
        }
        let mut data = vec![first_row];

        for line in liter {
            let row = parse_row(line);
            if row.len() != width {
                return Err(AocError::InvalidInput(format!(
                    "Map row '{}' has a length different from {}",
                    line, width
                )));
            }
            data.push(row);
        }
        Ok(Area {
            width,
            height: data.len(),
            data,
        })
    }
}

impl Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.data
                .iter()
                .map(|row| { row.iter().map(|s| -> char { s.into() }).collect::<String>() })
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl Simulator for Area {
    fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn get(&self, point: (usize, usize)) -> Seat {
        self.data[point.1][point.0]
    }

    fn set(&mut self, point: (usize, usize), val: Seat) {
        self.data[point.1][point.0] = val;
    }
}

impl SimulatorPart<PartA> for Area {
    fn point_occupied(&self, point: (usize, usize)) -> u32 {
        let range = |v: usize, b: usize| min(v.saturating_sub(1), b)..min(b, v + 2);
        iproduct!(range(point.0, self.width), range(point.1, self.height)).filter_count(
            |(sx, sy)| {
                !(*sx == point.0 && *sy == point.1) && matches!(self.data[*sy][*sx], Seat::Occupied)
            },
        )
    }
}

impl SimulatorPart<PartB> for Area {
    fn point_occupied(&self, point: (usize, usize)) -> u32 {
        todo!()
    }
}

impl Area {
    fn occupied(&self) -> u64 {
        self.data
            .iter()
            .flat_map(|row| row.iter())
            .filter_count(|s| matches!(s, Seat::Occupied))
    }
}

pub const SOLUTION: Solution = Solution {
    day: 11,
    name: "Seating System",
    solver: |input| {
        // Generation
        let area: Area = input.parse()?;

        // Process
        let answers = vec![match area.simulate() {
            SimulationStatus::Stable(a) => a.occupied(),
            SimulationStatus::Infinite(_) => {
                return Err(AocError::Process(
                    "Simulation did not reach a steady state".to_string(),
                ));
            }
        }];
        Ok(answers)
    },
};
