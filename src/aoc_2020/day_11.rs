use super::super::aoc::{AocError, Solution};
use itertools::iproduct;
use std::{cmp::min, collections::HashSet, convert::TryInto, fmt::Display, str::FromStr};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![0],
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

#[derive(Clone)]
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

#[derive(Clone)]
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

enum SimulationStatus {
    Stable(Area),
    Infinite(Area),
}

impl Area {
    fn adjacent_occupied(&self, x: usize, y: usize) -> u32 {
        let range = |v: usize, b: usize| min((v - 1).saturating_sub(1), b)..min(b, v + 2);
        iproduct!(range(x, self.width), range(y, self.height))
            .count()
            .try_into()
            .unwrap()
    }

    fn next(&self) -> Area {
        let next = self.clone();

        next
    }

    fn simulate(&self) -> SimulationStatus {
        SimulationStatus::Stable(self.next())
    }
}

struct Simulation {
    previous_states: HashSet<Area>,
}

pub const SOLUTION: Solution = Solution {
    day: 11,
    name: "Seating System",
    solver: |input| {
        // Generation
        let area: Area = input.parse()?;

        println!("{}", area);

        // Process
        let answers = vec![];
        Ok(answers)
    },
};
