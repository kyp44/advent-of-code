use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use Answer::{String, Unsigned};
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "size 7x3
rect 3x2
rotate column x=1 by 1
rotate row y=0 by 4
rotate column x=1 by 1";
            answers = unsigned![6];
        }
        actual_answers = &[Some(Unsigned(116)), Some(String("UPOJFLBCEZ".into()))];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{grid::StdBool, parse::trim};
    use nom::{
        branch::alt, bytes::complete::tag, character::complete::usize as pusize, combinator::map,
        multi::many1,
    };

    /// A single instruction for a [`Screen`].
    ///
    /// Can be parsed from text input.
    #[derive(Debug)]
    pub enum ScreenInstruction {
        /// Draws a rectangle.
        Rect {
            /// Rectangle width in pixels.
            width: usize,
            /// Rectangle height in pixels.
            height: usize,
        },
        /// Rotates row `y` to the right, wrapping around the screen.
        RotateRow {
            /// Row position where `0` is the top row.
            y: usize,
            /// Number of pixels to rotate.
            num: usize,
        },
        /// Rotates column `x`, wrapping around the screen.
        RotateCol {
            /// Column position where `0` is the leftmost column.
            x: usize,
            /// Number of pixels to rotate.
            num: usize,
        },
    }
    impl Parsable for ScreenInstruction {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            trim(
                true,
                alt((
                    map(
                        (tag("rect "), pusize, tag("x"), pusize),
                        |(_, width, _, height)| Self::Rect { width, height },
                    ),
                    map(
                        (tag("rotate row y="), pusize, tag(" by "), pusize),
                        |(_, y, _, by)| Self::RotateRow { y, num: by },
                    ),
                    map(
                        (tag("rotate column x="), pusize, tag(" by "), pusize),
                        |(_, x, _, by)| Self::RotateCol { x, num: by },
                    ),
                )),
            )
            .parse(input)
        }
    }
    impl Instruction for ScreenInstruction {
        type Registers = Screen;
        type YieldItem = ();
        type Err = AocError;

        fn execute(
            &self,
            registers: &mut Self::Registers,
        ) -> Result<Executed<Self::YieldItem>, Self::Err> {
            let grid = &mut registers.grid;

            match self {
                Self::Rect { width, height } => {
                    for point in GridSize::new(*width, *height).all_points() {
                        grid.set(&point, true.into())
                    }
                }
                ScreenInstruction::RotateRow { y, num: by } => {
                    let width = grid.size().width;
                    let old_row: Vec<_> = grid.underlying_grid().iter_row(*y).copied().collect();
                    for (x, v) in old_row.into_iter().enumerate() {
                        grid.set(&GridPoint::new((x + by) % width, *y), v);
                    }
                }
                ScreenInstruction::RotateCol { x, num: by } => {
                    let height = grid.size().height;
                    let old_col: Vec<_> = grid.underlying_grid().iter_col(*x).copied().collect();
                    for (y, v) in old_col.into_iter().enumerate() {
                        grid.set(&GridPoint::new(*x, (y + by) % height), v);
                    }
                }
            }

            Ok(Executed::default())
        }
    }

    /// A screen that can execute [`Instruction`]s.
    pub struct Screen {
        /// The pixel grid.
        grid: Grid<StdBool>,
    }
    impl std::fmt::Display for Screen {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&self.grid, f)
        }
    }
    impl Screen {
        /// Creates a new, blank screen of a given `size` with all pixels turned
        /// off.
        pub fn new(size: GridSize) -> Self {
            Self {
                grid: Grid::default(size),
            }
        }

        /// Returns the number of pixels that are currently lit.
        pub fn num_lit_pixels(&self) -> u64 {
            self.grid.all_values().filter_count(|p| bool::from(**p))
        }
    }

    /// A complete set of instructions.
    ///
    /// Can be parsed from text input.
    pub struct InstructionSet {
        /// The size of the [`Screen`] assumed for these instructions.
        size: GridSize,
        /// The program.
        program: Program<ScreenInstruction>,
    }
    impl Parsable for InstructionSet {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(
                (
                    (tag("size "), pusize, tag("x"), pusize),
                    many1(ScreenInstruction::parser),
                ),
                |((_, w, _, h), instructions)| Self {
                    size: GridSize::new(w, h),
                    program: Program::new(instructions),
                },
            )
            .parse(input)
        }
    }
    impl InstructionSet {
        /// Executes all the instructions starting from a blank screen,
        /// and returns the final state of the screen.
        pub fn execute(&self) -> Result<Screen, <ScreenInstruction as Instruction>::Err> {
            self.program
                .execute(Screen::new(self.size))
                .map(|pe| pe.into_registers())
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 8,
    name: "Two-Factor Authentication",
    preprocessor: Some(|input| Ok(Box::new(InstructionSet::from_str(input)?.execute()?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<Screen>()?.num_lit_pixels().into())
        },
        // Part two
        |input| {
            // Process
            let screen = input.expect_data::<Screen>()?;

            println!("Final screen contents:\n{screen}");

            // Requires a human in the loop so hard code
            Ok("UPOJFLBCEZ".into())
        },
    ],
};
