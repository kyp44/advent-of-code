use crate::aoc::{AocError, FilterCount, Solution};
use itertools::iproduct;
use std::{collections::HashSet, convert::TryInto, fmt::Display, hash::Hash, str::FromStr};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![2483, 2285],
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
        vec![37, 26]
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
    fn size(&self) -> (isize, isize);
    fn get(&self, point: &(isize, isize)) -> Option<Seat>;
    fn set(&mut self, point: &(isize, isize), val: Seat);

    fn valid_point(&self, point: &(isize, isize)) -> Option<(usize, usize)> {
        let size = self.size();
        if (0..size.0).contains(&point.0) && (0..size.1).contains(&point.1) {
            return Some((
                (*point).0.try_into().unwrap(),
                (*point).1.try_into().unwrap(),
            ));
        }
        None
    }
}

trait SimulatorPart<T: Part>
where
    Self: Sized + Clone + Simulator + Hash + Eq + Display,
{
    fn point_occupied(&self, point: &(isize, isize)) -> u32;

    fn next(&self) -> Self {
        let mut next = self.clone();
        let size = self.size();

        for (x, y) in iproduct!(0..size.0, 0..size.1) {
            let occupied = self.point_occupied(&(x, y));
            let orig = self.get(&(x, y)).unwrap();
            //println!("Seat ({}, {}) has {} around it", x, y, occupied);
            next.set(
                &(x, y),
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
    fn size(&self) -> (isize, isize) {
        (
            self.width.try_into().unwrap(),
            self.height.try_into().unwrap(),
        )
    }

    fn get(&self, point: &(isize, isize)) -> Option<Seat> {
        self.valid_point(point).map(|(x, y)| self.data[y][x])
    }

    fn set(&mut self, point: &(isize, isize), val: Seat) {
        if let Some((x, y)) = self.valid_point(point) {
            self.data[y][x] = val;
        }
    }
}

impl SimulatorPart<PartA> for Area {
    fn point_occupied(&self, point: &(isize, isize)) -> u32 {
        iproduct!((point.1 - 1)..=(point.1 + 1), (point.0 - 1)..=(point.0 + 1)).filter_count(
            |(sy, sx)| {
                !(*sx == point.0 && *sy == point.1)
                    && matches!(self.get(&(*sx, *sy)), Some(Seat::Occupied))
            },
        )
    }
}

impl SimulatorPart<PartB> for Area {
    fn point_occupied(&self, point: &(isize, isize)) -> u32 {
        struct Traverser<'a> {
            area: &'a Area,
            point: (isize, isize),
            direction: (isize, isize),
            stop: bool,
        }
        impl Traverser<'_> {
            fn new(area: &Area, point: (isize, isize), direction: (isize, isize)) -> Traverser {
                Traverser {
                    area,
                    point,
                    direction,
                    stop: false,
                }
            }
        }
        impl Iterator for Traverser<'_> {
            type Item = Seat;

            fn next(&mut self) -> Option<Self::Item> {
                if self.stop {
                    return None;
                }

                self.point = (
                    self.point.0 + self.direction.0,
                    self.point.1 + self.direction.1,
                );

                self.area.get(&self.point).map(|s| {
                    self.stop = matches!(s, Seat::Empty) || matches!(s, Seat::Occupied);
                    s
                })
            }
        }

        iproduct!(-1isize..=1, -1isize..=1)
            .filter(|(dx, dy)| !(*dx == 0 && *dy == 0))
            .map(|(dx, dy)| Traverser::new(self, *point, (dx, dy)).last())
            .filter_count(|s| matches!(s, Some(Seat::Occupied)))
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
        fn check_simulation(status: SimulationStatus<Area>) -> Result<u64, AocError> {
            match status {
                SimulationStatus::Stable(a) => Ok(a.occupied()),
                SimulationStatus::Infinite(_) => Err(AocError::Process(
                    "Simulation did not reach a steady state".to_string(),
                )),
            }
        }
        let answers = vec![
            check_simulation(SimulatorPart::<PartA>::simulate(&area))?,
            check_simulation(SimulatorPart::<PartB>::simulate(&area))?,
        ];
        Ok(answers)
    },
};
