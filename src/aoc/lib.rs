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
        NoYear(u16),
        /// The day has not been solved.
        #[error("Day {0} is not yet solved")]
        NoDay(u8),
        /// The day is out of range.
        #[error("Day {0} is not in the range of {} to {}", .1.start(), .1.end())]
        DayRange(u8, RangeInclusive<u8>),
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

    /// Extension trait for inclusive ranges.
    pub trait RangeExt<T>: Sized {
        /// Returns the number of discrete elements or steps in the range.
        ///
        /// # Examples
        /// Basic usage:
        /// ```
        /// # use aoc::prelude::*;
        /// assert_eq!((0..=5).size(), 6);
        /// assert_eq!((-3..=3).size(), 7);
        /// assert_eq!((4..=-7).size(), 0);
        /// assert_eq!((6..=6).size(), 1);
        /// ```
        fn size(&self) -> T;

        /// Returns the intersection of two ranges if they are not disjoint.
        ///
        /// # Examples
        /// Basic usage:
        /// ```
        /// # use aoc::prelude::*;
        /// assert_eq!((-4..=3).intersection(&(5..=9)), None);
        /// assert_eq!((0..=5).intersection(&(10..=2)), None);
        /// assert_eq!((0..=5).intersection(&(2..=10)), Some(2..=5));
        /// assert_eq!((-5..=10).intersection(&(-19..=-3)), Some(-5..=-3));
        /// assert_eq!((-5..=10).intersection(&(-2..=1)), Some(-2..=1));
        /// ```
        fn intersection(&self, other: &Self) -> Option<Self>;
    }
    impl<T> RangeExt<T> for RangeInclusive<T>
    where
        T: Integer + Copy,
    {
        fn size(&self) -> T {
            if self.end() >= self.start() {
                *self.end() - *self.start() + T::one()
            } else {
                T::zero()
            }
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
        /// Unsigned number.
        Unsigned(u64),
        /// Signed number.
        Signed(i64),
        /// Text.
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
        /// A string input.
        Text(&'a str),
        /// Pre-parsed data of some kind.
        Data(Box<dyn Any>),
    }
    impl<'a> SolverInput<'a> {
        /// Returns the string input if selected, otherwise an [`AocError::InvalidInput`].
        ///
        /// # Examples
        /// Basic usage:
        /// ```
        /// # #![feature(assert_matches)]
        /// # use std::assert_matches::assert_matches;
        /// # use aoc::prelude::*;
        /// assert_eq!(SolverInput::Text("test").expect_input().unwrap(), "test");
        /// assert_matches!(SolverInput::Data(Box::new(7)).expect_input(), Err(AocError::InvalidInput(_)));
        /// ```
        pub fn expect_input(&self) -> AocResult<&'a str> {
            if let Self::Text(s) = self {
                Ok(s)
            } else {
                Err(AocError::InvalidInput(
                    "Expected string input but got something else".into(),
                ))
            }
        }

        /// Returns the data input of a particular type if selected and the data is the correct type,
        /// otherwise an [`AocError::InvalidInput`].
        ///
        /// # Examples
        /// Basic usage:
        /// ```
        /// # #![feature(assert_matches)]
        /// # use std::assert_matches::assert_matches;
        /// # use aoc::prelude::*;
        /// assert_eq!(SolverInput::Data(Box::new(6u8)).expect_data::<u8>().unwrap(), &6);
        /// assert_matches!(SolverInput::Text("text").expect_data::<u8>(), Err(AocError::InvalidInput(_)));
        /// assert_matches!(SolverInput::Data(Box::new(6u16)).expect_data::<u8>(), Err(AocError::InvalidInput(_)));
        /// ```
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
    /// Converts text to [`SolverInput::Text`].
    impl<'a> From<&'a str> for SolverInput<'a> {
        fn from(value: &'a str) -> Self {
            Self::Text(value)
        }
    }
    /// Converts boxed data to [`SolverInput::Data`].
    impl<T: Any> From<Box<T>> for SolverInput<'_> {
        fn from(value: Box<T>) -> Self {
            Self::Data(value)
        }
    }

    /// A solver function for any parts of a day's problem.
    ///
    /// Solvers will either return an [`Answer`] or an [`AocError`] if there is some kind of problem.
    pub type SolverFunc = fn(&SolverInput) -> AocResult<Answer>;

    /// The solution for a day's problem.
    pub struct Solution {
        /// The day of the problem (1 to 25).
        pub day: u8,
        /// The name of the day's problem.
        pub name: &'static str,
        /// An optional preprocessing function to parse the input text and possibly perform
        /// other preprocessing only once.
        ///
        /// The output of this will be passed to all solvers as their input.
        /// If not preprocessor is set, the raw problem input will be passed to all solvers.
        /// This may also return an [`AocError`] if a problem is encountered.
        pub preprocessor: Option<fn(&str) -> AocResult<SolverInput>>,
        /// Solve functions for each part of the day's problem.
        pub solvers: &'static [SolverFunc],
    }
    impl Solution {
        /// Constructs a nice title from the day and name.
        pub fn title(&self) -> String {
            format!("Day {}: {}", self.day, self.name)
        }

        /// Runs the preprocessing function if applicable with the `input` text.
        ///
        /// If no preprocessor is set, the `input` is just returned wrapped in a [`SolverInput::Text`].
        pub fn preprocess<'a>(&self, input: &'a str) -> AocResult<SolverInput<'a>> {
            if let Some(pf) = self.preprocessor {
                pf(input)
            } else {
                Ok(input.into())
            }
        }

        /// Reads the input from the text file, runs the preprocessor if set, then runs the solvers
        /// and prints their answers.
        ///
        /// If the preprocessor or any of the solvers return an [`AocError`], further processing will
        /// stop and this will be returned. Otherwise the list of answers corresponding to each solver
        /// are returned.
        pub fn run_and_print(&self, year: u16) -> anyhow::Result<Vec<Answer>> {
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

    /// Package of solutions for a year's problems.
    pub struct YearSolutions {
        /// Year.
        pub year: u16,
        /// The solutions for each day's problem for this year.
        pub solutions: &'static [Solution],
    }
    impl YearSolutions {
        /// Retrieves the [`Solution`] for a day, if it exists.
        pub fn get_day(&self, day: u8) -> Option<&Solution> {
            self.solutions.iter().find(|s| s.day == day)
        }

        /// Returns the list of the day's title solutions for every day as a newline-delimited
        /// string.
        pub fn solution_list(&self) -> String {
            self.solutions
                .iter()
                .map(|solution| solution.title())
                .join("\n")
        }
    }

    /// Convenience trait to convert a list of answers types for use in tests
    pub trait AnswerVec {
        /// Converts a [`Vec`] of answer types into a [`Vec`] of [`Option::Some`] with the [`Answer`].
        ///
        /// # Examples
        /// Basic usage:
        /// ```
        /// # use aoc::prelude::*;
        /// assert_eq!(
        ///     vec![3u64, 4, 5].answer_vec(),
        ///     vec![
        ///         Some(Answer::Unsigned(3)),
        ///         Some(Answer::Unsigned(4)),
        ///         Some(Answer::Unsigned(5)),
        ///     ],
        /// );
        /// assert_eq!(
        ///     vec![-3i64, 4, -5].answer_vec(),
        ///     vec![
        ///         Some(Answer::Signed(-3)),
        ///         Some(Answer::Signed(4)),
        ///         Some(Answer::Signed(-5)),
        ///     ],
        /// );
        /// assert_eq!(
        ///     vec!["test1", "test2", "test3"].answer_vec(),
        ///     vec![
        ///         Some(Answer::String("test1".to_string())),
        ///         Some(Answer::String("test2".to_string())),
        ///         Some(Answer::String("test3".to_string())),
        ///     ],
        /// );
        /// ```
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

    /// Compares solution results with a vector.
    ///
    /// This typically is not used directly, but rather by the [`solution_test`]
    /// and [`expensive_test`] macros, and always in the context of a day's solution
    /// module in which there is a constant [`Solution`] structure called `SOLUTION`
    /// in the same scope. The `$input` should then be a static `&str` to pass as input
    /// to the solvers, and the `$answers` should be be a [`Vec<Option<Answer>>`] of the
    /// answers for each part for that `$input`.
    #[macro_export]
    macro_rules! solution_results {
        ($input: expr, $answers: expr) => {
            let vans: Vec<Option<Answer>> = $answers;

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
    (
        $(example = {
            input = $input: expr;
            answers = $exp: expr;
        })+
        actual_answers = $actual: expr;
    ) => {
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
