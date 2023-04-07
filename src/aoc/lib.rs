//! Support crate for the [Advent of Code solutions](../advent_of_code/index.html).
//!
//! Contains useful abstractions are that are used for one more than one solution.
#![feature(slice_pattern)]
#![feature(assert_matches)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use cgmath::{Point2, Point3, Vector2, Vector3};

pub mod evolver;
pub mod grid;
pub mod iter;
pub mod parse;

/// The prelude.
pub mod prelude {
    pub use super::{
        error::{AocError, AocResult},
        evolver::Evolver,
        extension::{PointExt, RangeExt, VectorExt},
        grid::{
            AnyGridPoint, AnyGridPointExt, FromGridStr, Grid, GridDefault, GridPoint, GridSize,
            GridSizeExt,
        },
        iter::{IteratorExt, StrExt},
        parse::{BitInput, DiscardInput, NomParseError, NomParseResult, Parseable, Sections},
        solution::{Answer, AnswerVec, Solution, SolverInput, YearSolutions},
        PointFrom, PointInto, TryPointFrom, TryPointInto,
    };
}

/// General errors.
pub mod error {
    use crate::parse::NomParseError;
    use std::borrow::Cow;
    use std::ops::RangeInclusive;
    use thiserror::Error;

    /// Errors that can occur in general and solution functions.
    #[derive(Debug, Clone, Error)]
    pub enum AocError {
        /// The year has not been solved.
        #[error("Year {0} is not yet solved")]
        NoYear(u32),
        /// The day has not been solved.
        #[error("Day {0} is not yet solved")]
        NoDay(u32),
        /// The day is out of range.
        #[error("Day {0} is not in the range of {} to {}", .1.start(), .1.end())]
        DayRange(u32, RangeInclusive<u32>),
        /// Could not parse the problem input.
        #[error("Could not parse input")]
        NomParse(
            #[source]
            #[from]
            NomParseError,
        ),
        /// Invalid problem input.
        #[error("Invalid input: {0}")]
        InvalidInput(Cow<'static, str>),
        /// Error while processing the solution.
        #[error("Error while processing: {0}")]
        Process(Cow<'static, str>),
        /// No solution found.
        #[error("No solution found!")]
        NoSolution,
    }

    /// Result with an [`AocError`].
    pub type AocResult<T> = Result<T, AocError>;
}

// TODO: Should we abstract exhaustively solving game states to achieve some metric?
// Will need to go through and see where this could be used prior to designing such
// trait to ensure that all use cases can be satisfied.

/// Extension traits.
pub mod extension {
    use std::ops::RangeInclusive;

    use cgmath::{Point2, Point3, Vector2, Vector3, Zero};
    use num::{Integer, Signed};

    /// Extension trait for mathematical vectors from [`cgmath`].
    pub trait VectorExt<T> {
        /// Calculates the [Manhattan length](https://en.wikipedia.org/wiki/Taxicab_geometry) of the vector.
        ///
        /// # Examples
        /// Basic usage:
        /// ```
        /// # use aoc::prelude::*;
        /// # use cgmath::{Vector2, Vector3};
        /// assert_eq!(Vector2::new(0, 0).manhattan_len(), 0);
        /// assert_eq!(Vector2::new(3, -10).manhattan_len(), 13);
        /// assert_eq!(Vector3::new(-5, 2, -4).manhattan_len(), 11);
        /// ```
        fn manhattan_len(&self) -> T;
    }
    impl<T> VectorExt<T> for Vector2<T>
    where
        T: Signed,
    {
        fn manhattan_len(&self) -> T {
            self.x.abs() + self.y.abs()
        }
    }
    impl<T> VectorExt<T> for Vector3<T>
    where
        T: Signed,
    {
        fn manhattan_len(&self) -> T {
            self.x.abs() + self.y.abs() + self.z.abs()
        }
    }

    /// Extension trait for mathematical points from [`cgmath`].
    pub trait PointExt {
        /// Returns the origin point of the coordinate system, that is the point
        /// with all zero components.
        ///
        /// # Examples
        /// Basic usage:
        /// ```
        /// # use aoc::prelude::*;
        /// # use cgmath::{Point2, Point3};
        /// assert_eq!(Point2::origin(), Point2::new(0, 0));
        /// assert_eq!(Point3::origin(), Point3::new(0, 0, 0));
        /// ```
        fn origin() -> Self;
    }
    impl<T: Zero> PointExt for Point2<T> {
        fn origin() -> Self {
            Self::new(T::zero(), T::zero())
        }
    }
    impl<T: Zero> PointExt for Point3<T> {
        fn origin() -> Self {
            Self::new(T::zero(), T::zero(), T::zero())
        }
    }

    // Extension trait for ranges.
    pub trait RangeExt<T>: Sized {
        fn len(&self) -> T;
        fn intersection(&self, other: &Self) -> Option<Self>;
    }
    impl<T> RangeExt<T> for RangeInclusive<T>
    where
        T: Integer + Copy,
    {
        fn len(&self) -> T {
            *self.end() - *self.start() + T::one()
        }

        fn intersection(&self, other: &Self) -> Option<Self> {
            let range = *self.start().max(other.start())..=*self.end().min(other.end());
            if self.is_empty() || other.is_empty() || range.is_empty() {
                None
            } else {
                Some(range)
            }
        }
    }
}

/// Types and utilities for implementing problem solutions.
pub mod solution {
    use std::{any::Any, fs};

    use anyhow::Context;
    use colored::Colorize;
    use itertools::Itertools;

    use crate::prelude::{AocError, AocResult};

    /// Different types of answers to problems.
    #[derive(Debug, PartialEq, Eq)]
    pub enum Answer {
        Unsigned(u64),
        Signed(i64),
        String(String),
    }
    impl From<u64> for Answer {
        fn from(n: u64) -> Self {
            Answer::Unsigned(n)
        }
    }
    impl From<i64> for Answer {
        fn from(n: i64) -> Self {
            Answer::Signed(n)
        }
    }
    impl From<String> for Answer {
        fn from(s: String) -> Self {
            Answer::String(s)
        }
    }

    /// Represents data that can be passed to a solver function.
    pub enum SolverInput<'a> {
        Text(&'a str),
        Data(Box<dyn Any>),
    }
    impl<'a> SolverInput<'a> {
        pub fn expect_input(&self) -> AocResult<&'a str> {
            if let Self::Text(s) = self {
                Ok(s)
            } else {
                Err(AocError::InvalidInput(
                    "Expected string input but got something else".into(),
                ))
            }
        }

        pub fn expect_data<T: 'static>(&self) -> AocResult<&T> {
            if let Self::Data(obj) = self {
                obj.downcast_ref::<T>().ok_or(AocError::InvalidInput(
                    "Expected data of one type but got a different type".into(),
                ))
            } else {
                Err(AocError::InvalidInput(
                    "Expected data input but got something else".into(),
                ))
            }
        }
    }
    impl<'a> From<&'a str> for SolverInput<'a> {
        fn from(value: &'a str) -> Self {
            Self::Text(value)
        }
    }
    impl<T: Any> From<Box<T>> for SolverInput<'_> {
        fn from(value: Box<T>) -> Self {
            Self::Data(value)
        }
    }

    // Represents the solver for both pars of a day's puzzle.
    type SolverFunc = fn(&SolverInput) -> AocResult<Answer>;
    pub struct Solution {
        pub day: u32,
        pub name: &'static str,
        pub preprocessor: Option<fn(&str) -> AocResult<SolverInput>>,
        pub solvers: &'static [SolverFunc],
    }
    impl Solution {
        // Constructs the title.
        pub fn title(&self) -> String {
            format!("Day {}: {}", self.day, self.name)
        }

        // Run preprocessor if applicable
        pub fn preprocess<'a>(&self, input: &'a str) -> AocResult<SolverInput<'a>> {
            if let Some(pf) = self.preprocessor {
                pf(input)
            } else {
                Ok(input.into())
            }
        }

        // Reads the input, runs the solvers, and outputs the answer(s).
        pub fn run_and_print(&self, year: u32) -> anyhow::Result<Vec<Answer>> {
            // Read input for the problem
            let input_path = format!("input/{year}/day_{:02}.txt", self.day);
            let input = fs::read_to_string(&input_path)
                .with_context(|| format!("Could not read input file {input_path}"))?;

            // Run solvers
            let data = self.preprocess(&input)?;
            let results = self
                .solvers
                .iter()
                .map(|s| s(&data))
                .collect::<AocResult<Vec<Answer>>>()?;

            println!("{}", format!("Year {} {}", year, self.title()).yellow());
            for (part, result) in ["one", "two"].into_iter().zip(results.iter()) {
                if results.len() > 1 {
                    println!("{}", format!("Part {part}:").bold().underline());
                }
                println!(
                    "Answer: {}",
                    match result {
                        Answer::Unsigned(n) => n.to_string(),
                        Answer::Signed(n) => n.to_string(),
                        Answer::String(s) => s.to_string(),
                    }
                );
            }

            Ok(results)
        }
    }

    // Package of solutions of a year's puzzles.
    pub struct YearSolutions {
        pub year: u32,
        pub solutions: &'static [Solution],
    }
    impl YearSolutions {
        pub fn get_day(&self, day: u32) -> Option<&Solution> {
            self.solutions.iter().find(|s| s.day == day)
        }

        pub fn solution_list(&self) -> String {
            self.solutions
                .iter()
                .map(|solution| solution.title())
                .join("\n")
        }
    }

    // Convenience trait to convert a vector of numbers into numeric answers.
    pub trait AnswerVec {
        fn answer_vec(self) -> Vec<Option<Answer>>;
    }
    impl AnswerVec for Vec<u64> {
        fn answer_vec(self) -> Vec<Option<Answer>> {
            self.into_iter()
                .map(|n| Some(Answer::Unsigned(n)))
                .collect()
        }
    }
    impl AnswerVec for Vec<i64> {
        fn answer_vec(self) -> Vec<Option<Answer>> {
            self.into_iter().map(|n| Some(Answer::Signed(n))).collect()
        }
    }
    impl AnswerVec for Vec<&str> {
        fn answer_vec(self) -> Vec<Option<Answer>> {
            self.into_iter()
                .map(|s| Some(Answer::String(s.into())))
                .collect()
        }
    }

    // Compares solution results with a vector.
    #[macro_export]
    macro_rules! solution_results {
        ($input:expr, $exp: expr) => {
            let vans: Vec<Option<Answer>> = $exp;

            let data = SOLUTION.preprocess($input).unwrap();

            for (solver, ans) in SOLUTION.solvers.iter().zip(vans.into_iter()) {
                if let Some(a) = ans {
                    assert_eq!(solver(&data).unwrap(), a);
                }
            }
        };
    }

    // Convenience macro to build the example test for a solution.
    // Also creates an ignored test to test the main problem solutions.
    #[macro_export]
    macro_rules! solution_test {
    ($actual: expr, $($input:expr, $exp: expr), +) => {
        #[test]
        #[ignore]
        fn actual() {
            assert_eq!(
                SOLUTION.run_and_print(super::super::YEAR_SOLUTIONS.year).unwrap(),
                $actual
            );
        }

        #[test]
        fn example() {
            use $crate::solution_results;
            $(
            solution_results!($input, $exp);
            )+
        }
    };
}

    // Builds expensive tests that take a while to run.
    #[macro_export]
    macro_rules! expensive_test {
    ($($input:expr, $exp: expr), +) => {
        #[test]
	    #[cfg(feature = "expensive")]
        fn expensive() {
            use $crate::solution_results;
                $(
            solution_results!($input, $exp);
                )+
        }
    };
}

    // Convenience macro to construct the solutions for a year.
    #[macro_export]
    macro_rules! year_solutions {
        (year = $year: expr, days =  {$($day: ident,)* }) => {
            $(
                pub mod $day;
            )*

            use aoc::solution::YearSolutions;

            // All of the solutions.
            pub const YEAR_SOLUTIONS: YearSolutions = YearSolutions {
                year: $year,
                solutions: &[
                $(
                    $day::SOLUTION,
                )*
                ],
            };
         }
    }
}

// TODO: Also need to try and look for cases where this might be useful, since it was largely forgotten about.
// TODO: Move this to a module maybe?
// convert module
// Cannot use std traits because of blanket conflicts.

/// Extension trait to convert between [`cgmath`] vector component types more easily.
///
/// Note that we cannot implement the [`std`] conversion traits due to the orphan rule.
///
/// # Examples
/// Basic usage:
/// ```
/// /* # #![feature(assert_matches)]
/// # use std::assert_matches::assert_matches;
/// # use aoc::prelude::*;
/// # use cgmath::{Vector2, Vector3};
/// // Some 2D vector conversions
/// assert_matches!(Vector2::<isize>::new(3, 4).try_point_into(), Ok(v) if v == Vector2::<usize>::new(3, 4));
/// assert_matches!(Vector2::<isize>::new(3, 4).try_point_into(), Ok(v) if v == Vector2::<u8>::new(3, 4));
/// assert_matches!(Vector2::<isize>::new(-3, 4).try_point_into(), Ok(v) if v == Vector2::<i8>::new(-3, 4));
/// assert_matches!(Vector2::<usize>::new(3, 4).try_point_into(), Ok(v) if v == Vector2::<u8>::new(3, 4));
/// assert_matches!(<Vector2<isize> as aoc::grid::PointTryInto<Vector2<usize>>>::try_point_into(Vector2::new(3, -4)), Err(_));
/// assert_matches!(<Vector2<isize> as aoc::grid::PointTryInto<Vector2<u8>>>::try_point_into(Vector2::new(3, -4)), Err(_));
/// assert_matches!(<Vector2<u16> as aoc::grid::PointTryInto<Vector2<u8>>>::try_point_into(Vector2::new(1000, 4)), Err(_));
/// assert_matches!(<Vector2<i64> as aoc::grid::PointTryInto<Vector2<i32>>>::try_point_into(Vector2::new(3, 4294967296)), Err(_));
///
/// // Some 3D vector conversions
/// assert_matches!(Vector3::<isize>::new(3, 4, 5).try_point_into(), Ok(v) if v == Vector3::<usize>::new(3, 4, 5));
/// assert_matches!(Vector3::<isize>::new(3, 4, 5).try_point_into(), Ok(v) if v == Vector3::<u8>::new(3, 4, 5));
/// assert_matches!(Vector3::<isize>::new(-3, 4, 5).try_point_into(), Ok(v) if v == Vector3::<i8>::new(-3, 4, 5));
/// assert_matches!(Vector3::<usize>::new(3, 4, 5).try_point_into(), Ok(v) if v == Vector3::<u8>::new(3, 4, 5));
/// assert_matches!(<Vector3<isize> as aoc::grid::PointTryInto<Vector3<usize>>>::try_point_into(Vector3::new(3, -4, 5)), Err(_));
/// assert_matches!(<Vector3<isize> as aoc::grid::PointTryInto<Vector3<u8>>>::try_point_into(Vector3::new(3, -4, 5)), Err(_));
/// assert_matches!(<Vector3<u16> as aoc::grid::PointTryInto<Vector3<u8>>>::try_point_into(Vector3::new(1000, 4, 5)), Err(_));
/// assert_matches!(<Vector3<i64> as aoc::grid::PointTryInto<Vector3<i32>>>::try_point_into(Vector3::new(3, 4294967296, 5)), Err(_)); */
/// ```
pub trait PointFrom<T> {
    fn point_from(value: T) -> Self;
}
pub trait PointInto<T> {
    fn point_into(self) -> T;
}
impl<T, S: PointFrom<T>> PointInto<S> for T {
    fn point_into(self) -> S {
        S::point_from(self)
    }
}

pub trait TryPointFrom<T>: Sized {
    type Error;

    fn try_point_from(value: T) -> Result<Self, Self::Error>;
}
pub trait TryPointInto<T> {
    type Error;

    fn try_point_into(self) -> Result<T, Self::Error>;
}
impl<T, S: TryPointFrom<T>> TryPointInto<S> for T {
    type Error = S::Error;

    fn try_point_into(self) -> Result<S, Self::Error> {
        S::try_point_from(self)
    }
}

macro_rules! impl_point_conversions {
    ($ArrayN:ident <$S:ident> {$($field:ident),+}) => {
        impl<T, $S: From<T>> PointFrom<$ArrayN<T>> for $ArrayN<$S> {
            fn point_from(value: $ArrayN<T>) -> Self {
                $ArrayN::new($(value.$field.into()),+)
            }
        }

        impl<T, $S: TryFrom<T>> TryPointFrom<$ArrayN<T>> for $ArrayN<$S> {
            type Error = S::Error;

            fn try_point_from(value: $ArrayN<T>) -> Result<Self, Self::Error> {
                Ok($ArrayN::new($(value.$field.try_into()?),+))
            }
        }
    };
}

impl_point_conversions!(Point2<S> {x, y});
impl_point_conversions!(Point3<S> {x, y, z});
impl_point_conversions!(Vector2<S> {x, y});
impl_point_conversions!(Vector3<S> {x, y, z});
