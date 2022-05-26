use std::convert::TryInto;

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
    fn parser(input: &str) -> NomParseResult<&str, Self> {
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
fn point_parser(input: &str) -> NomParseResult<&str, Point> {
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
    fn parser(input: &str) -> NomParseResult<&str, Self> {
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
    fn parser(input: &str) -> NomParseResult<&str, Self> {
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
#[generics(bool)]
struct LightGrid<T> {
    grid: Grid<T>,
}
impl<T: Clone + Part + Default> LightGrid<T> {
    fn new(size: usize) -> Self {
        LightGrid {
            grid: Grid::default(GridSize::new(size, size)),
        }
    }
}
impl<T: Part> LightGrid<T> {
    fn execute_instruction(&mut self, instructions: &[Instruction]) {
        for inst in instructions {
            for point in inst.rect.iter() {
                self.grid.element_at(&point).update(&inst.action);
            }
        }
    }
}
impl LightGrid<bool> {
    fn number_lit(&self) -> usize {
        self.grid.all_values().filter_count(|b| **b)
    }
}
impl CharGrid<bool> for LightGrid<bool> {
    fn get_grid(&self) -> &Grid<bool> {
        &self.grid
    }

    fn from_char(c: char) -> Option<bool> {
        match c {
            '#' => Some(true),
            '.' => Some(false),
            _ => None,
        }
    }

    fn to_char(e: &bool) -> char {
        if *e {
            '#'
        } else {
            '.'
        }
    }
}

impl LightGrid<u8> {
    fn total_brightness(&self) -> u64 {
        self.grid
            .all_values()
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
            //println!("{:?}", light_grid);

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
