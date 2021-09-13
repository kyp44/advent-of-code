use std::{convert::TryInto, fmt};

use cgmath::Vector2;
use itertools::iproduct;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space1,
    combinator::{map, map_opt, value},
    sequence::{separated_pair, tuple},
};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(543903), Unsigned(14687245)],
    "turn on 0,0 through 999,999
toggle 0,0 through 999,0
turn off 499,499 through 500,500",
    vec![Some(Unsigned(998996)), None],
    "turn on 0,0 through 0,0
    toggle 0,0 through 999,999",
    vec![None, Some(Unsigned(2000001))]
    }
}

#[derive(Clone)]
enum Action {
    TurnOn,
    Toggle,
    TurnOff,
}
impl Parseable<'_> for Action {
    fn parser(input: &str) -> NomParseResult<Self> {
        use Action::*;
        alt((
            value(TurnOn, tag("turn on")),
            value(Toggle, tag("toggle")),
            value(TurnOff, tag("turn off")),
        ))(input)
    }
}

type Point = Vector2<usize>;
// NOTE: This cannot be done as a Parseable implementation due
// to a potential conflict.
fn point_parser(input: &str) -> NomParseResult<Point> {
    use nom::character::complete::u64 as cu64;
    map(separated_pair(cu64, tag(","), cu64), |(x, y)| {
        Vector2::new(x.try_into().unwrap(), y.try_into().unwrap())
    })(input)
}

struct Rect {
    lower_left: Point,
    upper_right: Point,
}
impl Parseable<'_> for Rect {
    fn parser(input: &str) -> NomParseResult<Self> {
        map_opt(
            separated_pair(
                point_parser,
                tuple((space1, tag("through"), space1)),
                point_parser,
            ),
            |(ll, ur)| {
                if ll.x > ur.x || ll.y > ur.y {
                    None
                } else {
                    Some(Rect {
                        lower_left: ll,
                        upper_right: ur,
                    })
                }
            },
        )(input)
    }
}
impl Rect {
    fn iter(&self) -> impl Iterator<Item = Point> {
        iproduct!(
            self.lower_left.x..=self.upper_right.x,
            self.lower_left.y..=self.upper_right.y
        )
        .map(|(x, y)| Point::new(x, y))
    }
}

struct Instruction {
    action: Action,
    rect: Rect,
}
impl Parseable<'_> for Instruction {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(
            separated_pair(Action::parser, space1, Rect::parser),
            |(a, r)| Instruction { action: a, rect: r },
        )(input.trim())
    }
}

trait Part {
    fn initial() -> Self;
    fn update(&mut self, action: &Action);
}
impl Part for bool {
    fn initial() -> Self {
        false
    }

    fn update(&mut self, action: &Action) {
        use Action::*;
        match action {
            TurnOn => *self = true,
            Toggle => *self = !*self,
            TurnOff => *self = false,
        }
    }
}

impl Part for u8 {
    fn initial() -> Self {
        0
    }

    fn update(&mut self, action: &Action) {
        use Action::*;
        match action {
            TurnOn => *self += 1,
            Toggle => *self += 2,
            TurnOff => *self = self.saturating_sub(1),
        }
    }
}

#[derive(CharGridDebug)]
#[generics(<bool>)]
struct LightGrid<T> {
    grid: Box<[Box<[T]>]>,
}
impl<T: Clone + Part> LightGrid<T> {
    fn new(size: usize) -> Self {
        LightGrid {
            grid: vec![vec![T::initial(); size].into_boxed_slice(); size].into_boxed_slice(),
        }
    }
}
impl<T: Part> LightGrid<T> {
    fn execute_instruction(&mut self, instructions: &[Instruction]) {
        for inst in instructions {
            for point in inst.rect.iter() {
                self.grid[point.y][point.x].update(&inst.action);
            }
        }
    }
}
impl LightGrid<bool> {
    fn number_lit(&self) -> usize {
        self.grid
            .iter()
            .map(|row| row.iter())
            .flatten()
            .filter_count(|b| **b)
    }
}
impl CharGrid for LightGrid<bool> {
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
        Ok(LightGrid { grid: data })
    }

    fn to_data(&self) -> &[Box<[Self::Element]>] {
        &self.grid
    }
}

impl LightGrid<u8> {
    fn total_brightness(&self) -> u64 {
        self.grid
            .iter()
            .map(|row| row.iter())
            .flatten()
            .copied()
            .map::<u64, _>(|v| v.into())
            .sum()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 6,
    name: "Probably a Fire Hazard",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let mut light_grid = LightGrid::<bool>::new(1000);
            light_grid.execute_instruction(&Instruction::gather(input.lines())?);

            // Print the grid just to see what it is
            println!("{:?}", light_grid);

            // Process
            Ok(Answer::Unsigned(
                light_grid.number_lit().try_into().unwrap(),
            ))
        },
        // Part b)
        |input| {
            // Generation
            let mut light_grid = LightGrid::<u8>::new(1000);
            light_grid.execute_instruction(&Instruction::gather(input.lines())?);

            // Process
            Ok(light_grid.total_brightness().into())
        },
    ],
};
