use crate::aoc::prelude::*;
use itertools::iproduct;
use rayon::prelude::*;
use std::{collections::HashSet, convert::TryInto, fmt::Display, hash::Hash, str::FromStr};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Number;

    solution_test! {
    vec![Number(2483), Number(2285)],
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
        vec![37, 26].answer_vec()
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

trait Simulator
where
    Self: Sized,
{
    fn new(data: Vec<Vec<Seat>>) -> AocResult<Self>;
    fn size(&self) -> (isize, isize);
    fn get(&self, point: &(isize, isize)) -> Option<Seat>;

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
    Self: Sized + Clone + Simulator + Hash + Eq + Display + Sync,
{
    fn point_occupied(&self, point: &(isize, isize)) -> u32;

    fn next(&self) -> Self {
        let size = self.size();

        // Calculate new seats
        let data = (0..size.1)
            .into_par_iter()
            .map(|y| {
                (0..size.0)
                    .map(|x| {
                        let point = (x, y);
                        let occupied = self.point_occupied(&point);
                        let orig = self.get(&point).unwrap();
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
                        }
                    })
                    .collect()
            })
            .collect();

        Self::new(data).unwrap()
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
        fn parse_row(line: &str) -> Vec<Seat> {
            line.trim().chars().map(|c| c.into()).collect()
        }
        Area::new(s.lines().map(|line| parse_row(line)).collect())
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
    fn new(data: Vec<Vec<Seat>>) -> AocResult<Self> {
        // Verify the data
        let height = data.len();
        if height < 1 {
            return Err(AocError::InvalidInput(
                "Area vector has no rows!".to_string(),
            ));
        }
        let width = data[0].len();
        if width < 1 {
            return Err(AocError::InvalidInput(
                "First area row has no elements!".to_string(),
            ));
        }
        for (rn, row) in data.iter().enumerate() {
            if row.len() != width {
                return Err(AocError::InvalidInput(format!(
                    "Area row {} has an incorrect width of {} when it should be {}",
                    rn,
                    row.len(),
                    width
                )));
            }
        }
        Ok(Area {
            width,
            height,
            data,
        })
    }

    fn size(&self) -> (isize, isize) {
        (
            self.width.try_into().unwrap(),
            self.height.try_into().unwrap(),
        )
    }

    fn get(&self, point: &(isize, isize)) -> Option<Seat> {
        self.valid_point(point).map(|(x, y)| self.data[y][x])
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

fn check_simulation(status: SimulationStatus<Area>) -> AocResult<Answer> {
    match status {
        SimulationStatus::Stable(a) => Ok(a.occupied().into()),
        SimulationStatus::Infinite(_) => Err(AocError::Process(
            "Simulation did not reach a steady state".to_string(),
        )),
    }
}

pub const SOLUTION: Solution = Solution {
    day: 11,
    name: "Seating System",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let area: Area = input.parse()?;

            // Process
            check_simulation(SimulatorPart::<PartA>::simulate(&area))
        },
        // Part b)
        |input| {
            // Generation
            let area: Area = input.parse()?;

            // Process
            check_simulation(SimulatorPart::<PartB>::simulate(&area))
        },
    ],
};
