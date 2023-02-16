use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
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

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::grid::{Digit, StdBool};
    use cgmath::Point2;
    use itertools::iproduct;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::space1,
        combinator::{map, map_opt, value},
        sequence::{separated_pair, tuple},
    };

    /// An action that can occur for a light.
    #[derive(Clone)]
    pub enum Action {
        /// Turn the light on.
        TurnOn,
        /// Toggle the light.
        Toggle,
        /// Turn the light off.
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

    /// That that describes the position of a light.
    type Point = Point2<usize>;
    /// [`nom`] parser for a [`Point`].
    ///  
    /// NOTE: This cannot be done as a [`Parseable`] implementation due
    /// to a potential conflict.
    fn point_parser(input: &str) -> NomParseResult<&str, Point> {
        use nom::character::complete::u64 as cu64;
        map(separated_pair(cu64, tag(","), cu64), |(x, y)| {
            Point2::new(x, y).try_point_into().unwrap()
        })(input)
    }

    /// A Rectangle of lights that can be parsed from text input.
    struct Rect {
        /// Lower left corner of the rectangle (inclusive).
        lower_left: Point,
        /// Upper right corner of the rectangle (inclusive).
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
        /// Returns an [`Iterator`] of points contained in the rectangle.
        fn iter(&self) -> impl Iterator<Item = Point> {
            iproduct!(
                self.lower_left.x..=self.upper_right.x,
                self.lower_left.y..=self.upper_right.y
            )
            .map(|(x, y)| Point::new(x, y))
        }
    }

    /// Instruction to perform an action on a rectangle of lights.
    ///
    /// Can be parsed from text input.
    pub struct Instruction {
        /// Action to perform on all of the lights.
        action: Action,
        /// Rectangle of lights over which to perform the action.
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

    /// Light states particular to a part of the problem.
    pub trait Part {
        /// Initial state.
        fn initial() -> Self;
        /// Updates this light state based on the action.
        fn update(&mut self, action: &Action);
    }
    impl Part for StdBool {
        fn initial() -> Self {
            false.into()
        }

        fn update(&mut self, action: &Action) {
            use Action::*;
            *self = match action {
                TurnOn => true.into(),
                Toggle => !*self,
                TurnOff => false.into(),
            }
        }
    }

    impl Part for Digit {
        fn initial() -> Self {
            0.into()
        }

        fn update(&mut self, action: &Action) {
            use Action::*;
            match action {
                TurnOn => *self += 1.into(),
                Toggle => *self += 2.into(),
                TurnOff => *self = self.saturating_sub(1).into(),
            }
        }
    }

    /// Grid of lights.
    pub struct LightGrid<T> {
        /// Actual grid.
        grid: Grid<T>,
    }
    impl<T: Clone + Part + Default> LightGrid<T> {
        /// Creates a new square grid of lights of the given size.
        pub fn new(size: usize) -> Self {
            LightGrid {
                grid: Grid::default(GridSize::new(size, size)),
            }
        }
    }
    impl<T: Part> From<Grid<T>> for LightGrid<T> {
        fn from(value: Grid<T>) -> Self {
            Self { grid: value }
        }
    }
    impl<T: Part> LightGrid<T> {
        /// Executes a list of instructions on the given light grid.
        pub fn execute_instruction(&mut self, instructions: &[Instruction]) {
            for inst in instructions {
                for point in inst.rect.iter() {
                    self.grid.element_at(&point).update(&inst.action);
                }
            }
        }
    }

    impl LightGrid<StdBool> {
        /// Determines the number of lights that are lit.
        pub fn number_lit(&self) -> usize {
            self.grid.all_values().filter_count(|b| ***b)
        }
    }

    impl LightGrid<Digit> {
        /// Calculates the total brightness across all of the lights.
        pub fn total_brightness(&self) -> u64 {
            self.grid
                .all_values()
                .copied()
                .map::<u64, _>(|v| u64::from(*v))
                .sum()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 6,
    name: "Probably a Fire Hazard",
    preprocessor: Some(|input| Ok(Box::new(Instruction::gather(input.lines())?).into())),
    solvers: &[
        // Part one
        |input| {
            // Generation
            let mut light_grid = LightGrid::new(1000);
            light_grid.execute_instruction(input.expect_data::<Vec<Instruction>>()?);

            // Print the grid just to see what it is
            //println!("{:?}", light_grid);

            // Process
            Ok(Answer::Unsigned(
                light_grid.number_lit().try_into().unwrap(),
            ))
        },
        // Part two
        |input| {
            // Generation
            let mut light_grid = LightGrid::new(1000);
            light_grid.execute_instruction(input.expect_data::<Vec<Instruction>>()?);

            // Process
            Ok(light_grid.total_brightness().into())
        },
    ],
};
