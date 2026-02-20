use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use Answer::Unsigned;
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "turn on 0,0 through 999,999
toggle 0,0 through 999,0
turn off 499,499 through 500,500";
            answers = &[Some(Unsigned(998996)), None];
        }
        example {
            input = "turn on 0,0 through 0,0
    toggle 0,0 through 999,999";
            answers = &[None, Some(Unsigned(2000001))];
        }
        actual_answers = unsigned![543903, 14687245];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::grid::{Digit, StdBool};
    use derive_more::{From, Into};
    use euclid::point2;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::space1,
        combinator::{map, map_opt, value},
        sequence::separated_pair,
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
    impl Parsable for Action {
            fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            use Action::*;
            alt((
                value(TurnOn, tag("turn on")),
                value(Toggle, tag("toggle")),
                value(TurnOff, tag("turn off")),
            ))
            .parse(input)
        }
    }

    /// This is a [`nom`] parser for a [`GridPoint`].
    ///  
    /// NOTE: This cannot be done as a [`Parsable`] implementation due
    /// to a potential conflict.
    fn point_parser(input: &str) -> NomParseResult<&str, GridPoint> {
        use nom::character::complete::u64 as cu64;
        map(separated_pair(cu64, tag(","), cu64), |(x, y)| {
            point2(x, y).to_usize()
        })
        .parse(input)
    }

    /// A Rectangle of lights that can be parsed from text input.
    #[derive(Into)]
    pub struct ParseRect(GridBox);
    impl Parsable for ParseRect {
            fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map_opt(
                separated_pair(point_parser, (space1, tag("through"), space1), point_parser),
                |(ll, ur)| {
                    if ll.x > ur.x || ll.y > ur.y {
                        None
                    } else {
                        Some(ParseRect(GridBox::new_inclusive(ll, ur)))
                    }
                },
            )
            .parse(input)
        }
    }

    /// Instruction to perform an action on a rectangle of lights.
    ///
    /// Can be parsed from text input.
    pub struct Instruction {
        /// Action to perform on all of the lights.
        pub action: Action,
        /// Rectangle of lights over which to perform the action.
        pub rect: GridBox,
    }
    impl Parsable for Instruction {
            fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(
                separated_pair(Action::parser, space1, ParseRect::parser),
                |(a, r)| Instruction {
                    action: a,
                    rect: r.into(),
                },
            )
            .parse(input.trim())
        }
    }

    /// Light states particular to a part of the problem.
    pub trait Part {
        /// Updates this light state based on the action.
        fn update(&mut self, action: &Action);
    }
    impl Part for StdBool {
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
    #[derive(From)]
    pub struct LightGrid<T>(Grid<T>);
    impl<T: Clone + Part + Default> LightGrid<T> {
        /// Creates a new square grid of lights of the given size.
        pub fn new(size: usize) -> Self {
            LightGrid(Grid::default(GridSize::new(size, size)))
        }
    }
    impl<T: Part> LightGrid<T> {
        /// Executes a list of instructions on the given light grid.
        pub fn execute_instruction(&mut self, instructions: &[Instruction]) {
            for inst in instructions {
                for point in inst.rect.all_points() {
                    self.0.get_mut(&point).update(&inst.action);
                }
            }
        }
    }

    impl LightGrid<StdBool> {
        /// Determines the number of lights that are lit.
        pub fn number_lit(&self) -> usize {
            self.0.all_values().filter_count(|b| ***b)
        }
    }

    impl LightGrid<Digit> {
        /// Calculates the total brightness across all of the lights.
        pub fn total_brightness(&self) -> u64 {
            self.0
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
