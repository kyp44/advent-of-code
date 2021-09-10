use crate::aoc::prelude::*;
use cgmath::{Vector2, Zero};
use itertools::iproduct;
use std::{collections::HashSet, convert::TryInto, fmt::Display, hash::Hash, rc::Rc};

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
    Outside,
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
            Seat::Outside => 'O',
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

    fn point_occupied(&self, area: &Area, point: &Vector2<isize>) -> u8 {
        match self {
            Part::PartA => iproduct!(-1isize..=1, -1isize..=1).filter_count(|(dx, dy)| {
                let dp = Vector2::new(*dx, *dy);
                dp != Vector2::zero() && area.get(&(point + dp)) == Seat::Occupied
            }),
            Part::PartB => iproduct!(-1isize..=1, -1isize..=1)
                .map(|(dx, dy)| Vector2::new(dx, dy))
                .filter(|dp| *dp != Vector2::zero())
                .filter_count(|dp| {
                    let mut i = 1;
                    loop {
                        match area.get(&(point + i * dp)) {
                            Seat::Occupied => break true,
                            Seat::Empty => break false,
                            Seat::Outside => break false,
                            Seat::Floor => (),
                        }
                        i += 1;
                    }
                }),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq, CharGridDebug)]
struct Area {
    part: Part,
    size: Vector2<isize>,
    data: Box<[Box<[Seat]>]>,
}

impl CharGrid for Area {
    type Element = Seat;

    fn from_char(c: char) -> Self::Element {
        c.into()
    }

    fn to_char(e: &Self::Element) -> char {
        e.into()
    }

    fn from_data(size: (usize, usize), data: Box<[Box<[Self::Element]>]>) -> AocResult<Self>
    where
        Self: Sized,
    {
        Ok(Area {
            part: Part::PartA,
            size: Vector2::new(size.0.try_into().unwrap(), size.1.try_into().unwrap()),
            data,
        })
    }

    fn to_data(&self) -> &[Box<[Self::Element]>] {
        &self.data
    }
}

impl Evolver<Seat> for Area {
    type Point = Vector2<isize>;
    type Iter = impl Iterator<Item = Self::Point>;

    fn new(other: &Self) -> Self {
        Area {
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
        }
    }

    fn get(&self, point: &Self::Point) -> Seat {
        match self.point_usize(point) {
            None => Seat::Outside,
            Some(p) => self.data[p.y][p.x],
        }
    }

    fn set(&mut self, point: &Self::Point, value: Seat) {
        let point = self.point_usize(point).unwrap();
        self.data[point.y][point.x] = value;
    }

    fn next_cell(&self, point: &Self::Point) -> Seat {
        let occupied = self.part.point_occupied(self, point);
        let orig = self.get(point);
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

    fn next_iter(&self) -> Self::Iter {
        iproduct!(0..self.size.x, 0..self.size.y).map(|(x, y)| Vector2::new(x, y))
    }
}

impl Area {
    fn set_part(&mut self, part: Part) {
        self.part = part;
    }

    fn point_usize(&self, point: &Vector2<isize>) -> Option<Vector2<usize>> {
        if (0..self.size.x).contains(&point.x) && (0..self.size.y).contains(&point.y) {
            return Some(Vector2::new(
                point.x.try_into().unwrap(),
                point.y.try_into().unwrap(),
            ));
        }
        None
    }

    fn occupied(&self) -> u64 {
        self.next_iter()
            .filter_count(|point| matches!(self.get(point), Seat::Occupied))
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
