use crate::aoc::prelude::*;
use cgmath::{Vector2, Zero};
use itertools::iproduct;
use std::{collections::HashSet, fmt::Display, hash::Hash, rc::Rc, str::FromStr};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(2483), Unsigned(2285)],
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
        vec![37u64, 26].answer_vec()
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

impl Default for Seat {
    fn default() -> Self {
        Seat::Floor
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

/// Had to end up using an enum for this, which is not ideal
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Part {
    PartA,
    PartB,
}
impl Part {
    fn min_needed_to_vacate(&self) -> u8 {
        match self {
            Part::PartA => 4,
            Part::PartB => 5,
        }
    }

    /// Number of occupied seats for a given seat according to the rules for each part
    fn point_occupied(&self, area: &Area, point: &GridPoint) -> u8 {
        let area = &area.area;
        match self {
            Part::PartA => area
                .neighbor_points(point, true, false)
                .filter_count(|point| *area.get(point) == Seat::Occupied),
            Part::PartB => iproduct!(-1isize..=1, -1isize..=1)
                .map(|(dx, dy)| Vector2::new(dx, dy))
                .filter(|dp| *dp != Vector2::zero())
                .filter_count(|dp| {
                    let mut i = 1;
                    loop {
                        let point = Vector2::<isize>::new(
                            point.x.try_into().unwrap(),
                            point.y.try_into().unwrap(),
                        );

                        match area.valid_point(&(point + i * dp)) {
                            Some(p) => match area.get(&p) {
                                Seat::Occupied => break true,
                                Seat::Empty => break false,
                                Seat::Floor => (),
                            },
                            None => break false,
                        }

                        i += 1;
                    }
                }),
        }
    }
}

//TODO
#[derive(Clone, Hash, PartialEq, Eq, new)]
struct Area {
    part: Part,
    area: BasicGrid<Seat>,
}
impl CharGrid<Seat> for BasicGrid<Seat> {
    fn from_char(c: char) -> Option<Seat> {
        Some(c.into())
    }

    fn to_char(e: &Seat) -> char {
        e.into()
    }
}
impl FromStr for Area {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Area {
            part: Part::PartA,
            area: BasicGrid::from_str(s)?,
        })
    }
}

impl Evolver<Seat> for Area {
    type Point = GridPoint;

    /* TODO why did we map this?
        fn new(other: &Self) -> Self {
            let mut area = Area {
                part: other.part,
                size: other.size,
                data: other
                    .data
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|s| match s {
                                Seat::Empty => Seat::Empty,
                                Seat::Floor => Seat::Floor,
                                Seat::Occupied => Seat::Empty,
                                Seat::Outside => Seat::Outside,
                            })
                            .collect()
                    })
                    .collect(),
            };
    }*/

    fn new(other: &Self) -> Self {
        Area::new(other.part, BasicGrid::default(*other.area.size()))
    }

    fn get_element(&self, point: &Self::Point) -> Seat {
        *self.area.get(point)
    }

    fn set_element(&mut self, point: &Self::Point, value: Seat) {
        self.area.set(point, value)
    }

    fn next_cell(&self, point: &Self::Point) -> Seat {
        let occupied = self.part.point_occupied(self, point);
        let orig = self.get_element(point);
        match orig {
            Seat::Empty => {
                if occupied == 0 {
                    Seat::Occupied
                } else {
                    orig
                }
            }
            Seat::Occupied => {
                if occupied >= self.part.min_needed_to_vacate() {
                    Seat::Empty
                } else {
                    orig
                }
            }
            _ => orig,
        }
    }

    fn next_iter(&self) -> Box<dyn Iterator<Item = Self::Point>> {
        self.area.all_points()
    }
}

impl Area {
    fn set_part(&mut self, part: Part) {
        self.part = part;
    }

    fn occupied(&self) -> u64 {
        self.next_iter()
            .filter_count(|point| matches!(self.get_element(point), Seat::Occupied))
    }

    fn simulate(&self) -> SimulationStatus<Rc<Self>> {
        let mut prior_states: HashSet<Rc<Self>> = HashSet::new();
        let mut last_state = prior_states.get_or_insert(Rc::new(self.clone()));
        for state in self.evolutions() {
            //println!("{:?}", state);
            if state == *last_state {
                return SimulationStatus::Stable(state);
            }
            if prior_states.contains(&state) {
                return SimulationStatus::Infinite(state);
            }
            last_state = prior_states.get_or_insert(state);
        }
        panic!("Somehow the evolver iterator ended!")
    }
}

fn check_simulation(status: SimulationStatus<Rc<Area>>) -> AocResult<Answer> {
    match status {
        SimulationStatus::Stable(a) => Ok(a.occupied().into()),
        SimulationStatus::Infinite(_) => Err(AocError::Process(
            "Simulation did not reach a steady state".into(),
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
            let area = Area::from_str(input)?;

            // Process
            check_simulation(area.simulate())
        },
        // Part b)
        |input| {
            // Generation
            let mut area = Area::from_str(input)?;
            area.set_part(Part::PartB);

            // Process
            check_simulation(area.simulate())
        },
    ],
};
