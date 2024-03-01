//! Support crate for the [Advent of Code solutions](../advent_of_code/index.html).
//!
//! Contains useful abstractions are that are used for one more than one solution.
#![feature(slice_pattern)]
#![feature(assert_matches)]
#![warn(missing_docs)]
#![feature(let_chains)]
#![feature(associated_type_defaults)]
#![warn(clippy::missing_docs_in_private_items)]

pub mod evolver;
pub mod grid;
pub mod iter;
pub mod parse;
pub mod tree_search;

/// The prelude.
pub mod prelude {
    pub use super::{
        error::{AocError, AocResult},
        evolver::Evolver,
        extension::{PointFrom, PointInto, RangeExt, TryPointFrom, TryPointInto, VectorExt},
        grid::{
            AnyGridPoint, AnyGridPointExt, FromGridStr, Grid, GridDefault, GridPoint, GridSize,
            GridSizeExt,
        },
        iter::{IteratorExt, LendingIteratorExt, StrExt},
        parse::{BitInput, DiscardInput, NomParseError, NomParseResult, Parsable, Sections},
        solution::{Answer, Solution, SolverInput, YearSolutions},
    };
}

/// Prelude for the tests, mainly when using [`solution_tests`].
pub mod prelude_test {
    pub use super::{
        answers, signed, solution::Answer, solution_results, solution_tests, string, unsigned,
    };
}

/// General errors.
pub mod error {
    use crate::parse::NomParseError;
    use std::borrow::Cow;
    use std::ops::RangeInclusive;
    use thiserror::Error;

    /// Errors that can occur in general and solution functions.
    #[derive(Debug, Clone, Error, PartialEq, Eq)]
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

/// Collection of general extension traits.
pub mod extension {
    use cgmath::{Point2, Point3, Vector2, Vector3};
    use num::{Integer, Signed};
    use std::ops::RangeInclusive;

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

    /// Extension trait to convert between [`cgmath`] vector component types more easily.
    ///
    /// Together with [`PointInto`], these are analogous to the [`From`] and [`Into`] traits
    /// and the interaction between them. Note that we cannot implement these [`std`]
    /// conversion traits due to the orphan rule.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// # use cgmath::{Vector2, Vector3, Point2, Point3};
    /// let v: Vector2<i64> = Vector2::<u32>::new(3, 4).point_into();
    /// assert_eq!(v, Vector2::<i64>::new(3, 4));
    /// assert_eq!(
    ///     Vector3::<u16>::point_from(Vector3::<u8>::new(7, 14, 21)),
    ///     Vector3::<u16>::new(7, 14, 21)
    /// );
    /// let v: Point2<i32> = Point2::<i16>::new(-4, 9).point_into();
    /// assert_eq!(v, Point2::<i32>::new(-4, 9));
    /// assert_eq!(
    ///     Point3::<usize>::point_from(Point3::<u8>::new(6, 5, 4)),
    ///     Point3::<usize>::new(6, 5, 4)
    /// );
    /// ```
    pub trait PointFrom<T> {
        /// Converts the point into another point with a different component type.
        ///
        /// Refer to [`PointFrom`].
        fn point_from(value: T) -> Self;
    }

    /// Extension trait to convert between [`cgmath`] vector component types more easily.
    ///
    /// Refer to [`PointFrom`].
    pub trait PointInto<T> {
        /// Converts the point into another point with a different component type.
        ///
        /// Refer to [`PointFrom`].
        fn point_into(self) -> T;
    }

    impl<T, S: PointFrom<T>> PointInto<S> for T {
        /// Converts the point into another point with a different component type.
        ///
        /// Refer to [`PointFrom`].
        fn point_into(self) -> S {
            S::point_from(self)
        }
    }

    /// Extension trait to convert between [`cgmath`] vector component types more easily.
    ///
    /// Together with [`TryPointInto`], these are analogous to the [`TryFrom`] and [`TryInto`]
    /// traits and the interaction between them. Note that we cannot implement these [`std`]
    /// conversion traits due to the orphan rule.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # #![feature(assert_matches)]
    /// # use std::assert_matches::assert_matches;
    /// # use aoc::prelude::*;
    /// # use cgmath::{Vector2, Vector3, Point2, Point3};
    /// assert_eq!(
    ///     Vector2::<isize>::new(3, 4).try_point_into(),
    ///     Ok(Vector2::<usize>::new(3, 4))
    /// );
    /// assert_matches!(
    ///     Vector2::<usize>::try_point_from(Vector2::<isize>::new(3, -4)),
    ///     Err(_),
    /// );
    /// assert_eq!(
    ///     Vector3::<u64>::new(23, 255, 78).try_point_into(),
    ///     Ok(Vector3::<u8>::new(23, 255, 78)),
    /// );
    /// assert_matches!(
    ///     Vector3::<u8>::try_point_from(Vector3::<u64>::new(45, 2, 256)),
    ///     Err(_),
    /// );
    /// assert_eq!(
    ///     Point2::<i16>::new(-9, 5).try_point_into(),
    ///     Ok(Point2::<i8>::new(-9, 5)),
    /// );
    /// assert_matches!(
    ///     Point2::<i8>::try_point_from(Point2::<i16>::new(-1000, 4)),
    ///     Err(_),
    /// );
    /// assert_eq!(
    ///     Point3::<usize>::new(1000, 2000, 3000).try_point_into(),
    ///     Ok(Point3::<u16>::new(1000, 2000, 3000)),
    /// );
    /// assert_matches!(
    ///     Point3::<u8>::try_point_from(Point3::<usize>::new(0, 1, usize::MAX)),
    ///     Err(_),
    /// );
    /// ```
    pub trait TryPointFrom<T>: Sized {
        /// Error type, the same type as the [`TryFrom::Error`] for the component type.
        type Error;

        /// Tries to convert the point into another point with a different component type.
        ///
        /// Refer to [`TryPointFrom`].
        fn try_point_from(value: T) -> Result<Self, Self::Error>;
    }

    /// Extension trait to convert between [`cgmath`] vector component types more easily.
    ///
    /// Refer to [`TryPointFrom`].
    pub trait TryPointInto<T> {
        /// Error type, the same type as the [`TryInto::Error`] for the component type.
        type Error;

        /// Tries to convert the point into another point with a different component type.
        ///
        /// Refer to [`TryPointFrom`].
        fn try_point_into(self) -> Result<T, Self::Error>;
    }

    impl<T, S: TryPointFrom<T>> TryPointInto<S> for T {
        type Error = S::Error;

        /// Tries to convert the point into another point with a different component type.
        ///
        /// Refer to [`TryPointFrom`].
        fn try_point_into(self) -> Result<S, Self::Error> {
            S::try_point_from(self)
        }
    }

    /// Implements the [`PointFrom`] and [`TryPointFrom`] traits for a vector type.
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

        /// Returns whether this range totally contains another range.
        ///
        /// # Examples
        /// Basic usage:
        /// ```
        /// # use aoc::prelude::*;
        /// assert_eq!((-4..=3).contains_range(&(-2..=3)), true);
        /// assert_eq!((-4..=3).contains_range(&(-5..=2)), false);
        /// assert_eq!((-4..=3).contains_range(&(-2..=7)), false);
        /// assert_eq!((0..=8).contains_range(&(0..=8)), true);
        /// ```
        fn contains_range(&self, other: &Self) -> bool;
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

        fn contains_range(&self, other: &Self) -> bool {
            self.start() <= other.start() && other.end() <= self.end()
        }
    }
}

/// Types and utilities for implementing problem solutions.
pub mod solution {
    use std::{any::Any, borrow::Cow, fs};

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
        String(Cow<'static, str>),
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
    impl From<&'static str> for Answer {
        fn from(s: &'static str) -> Self {
            Answer::String(s.into())
        }
    }
    impl From<String> for Answer {
        fn from(s: String) -> Self {
            Answer::String(s.into())
        }
    }
    impl std::fmt::Display for Answer {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Answer::Unsigned(n) => n.fmt(f),
                Answer::Signed(n) => n.fmt(f),
                Answer::String(s) => s.fmt(f),
            }
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
        pub fn run_and_print(&self, year: u16) -> anyhow::Result<Vec<Option<Answer>>> {
            // Read input for the problem
            let input_path = format!("input/{year}/day_{:02}.txt", self.day);
            let input = fs::read_to_string(&input_path)
                .with_context(|| format!("Could not read input file {input_path}"))?;

            // Run solvers
            let data = self.preprocess(&input)?;
            let results = self
                .solvers
                .iter()
                .map(|s| Ok(Some(s(&data)?)))
                .collect::<AocResult<Vec<_>>>()?;

            println!("{}", format!("Year {} {}", year, self.title()).yellow());
            for (part, result) in ["one", "two"].into_iter().zip(results.iter()) {
                if results.len() > 1 {
                    println!("{}", format!("Part {part}:").bold().underline());
                }
                println!("Answer: {}", result.as_ref().unwrap());
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

    /// Macro to construct the solution table for a year.
    ///
    /// See an implemented year for usage example.
    #[macro_export]
    macro_rules! year_solutions {
        (
            year = $year: expr;
            days = [
                $($day: ident,)*
            ];
        ) => {
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

    /// Wraps elements in [`Option::Some`] and evaluates to an answer slice.
    ///
    /// This is mainly for use with the [`solution_results`](crate::solution_results) macro.
    #[macro_export]
    macro_rules! answers {
        [$($val: expr),+] => {
            &[$(Some($val),)+]
        };
    }

    /// Wraps elements in [`Answer::Unsigned`] and evaluates to answer slice.
    #[macro_export]
    macro_rules! unsigned {
        [$($val: expr),+] => {
            answers![$(Answer::Unsigned($val)),+]
        };
    }

    /// Wraps elements in [`Answer::Signed`] and evaluates to answer slice.
    #[macro_export]
    macro_rules! signed {
        [$($val: expr),+] => {
            answers![$(Answer::Signed($val)),+]
        };
    }

    /// Wraps elements in [`Answer::String`] and evaluates to answer slice.
    #[macro_export]
    macro_rules! string {
        [$($val: expr),+] => {
            answers![$(Answer::String($val.into())),+]
        };
    }

    /// Compares solution results with a vector.
    ///
    /// This typically is not used directly, but rather by the [`solution_tests`](crate::solution_tests)
    /// macro, and always in the context of a day's solution
    /// module in which there is a constant [`Solution`] structure called `SOLUTION`
    /// in the same scope. The `$input` should then be a static `&str` to pass as input
    /// to the solvers, and the `$answers` should be be a [`Vec<Option<Answer>>`] of the
    /// answers for each part for that `$input`.
    #[macro_export]
    macro_rules! solution_results {
        ($input: expr, $answers: expr) => {
            let vans: &[Option<Answer>] = $answers;
            let data = SOLUTION.preprocess($input).unwrap();

            for (solver, ans) in SOLUTION.solvers.iter().zip(vans.iter()) {
                if let Some(a) = ans {
                    assert_eq!(solver(&data).unwrap(), *a);
                }
            }
        };
    }

    /// Macro to build the tests for a solution.
    ///
    /// Creates zero or more example tests and also creates an ignored
    /// test to verify the solution with the actual input. Optionally,
    /// computationally expensive example tests can be created that are
    /// only executed when the `expensive` feature is enabled.
    ///
    /// Refer to the many implemented solutions for how to use this.
    /// For example, the 2015 Day 10 solution features all of these
    /// tests.
    #[macro_export]
    macro_rules! solution_tests {
        (
            $(example {
                input = $input: expr;
                answers = $answers: expr;
            })*
            $(expensive_example {
                input = $exp_input: expr;
                answers = $exp_answers: expr;
            })*
            actual_answers = $actual: expr;
        ) => {
            #[test]
            fn examples() {
                use super::SOLUTION;
                $(
                solution_results!($input, $answers);
                )*
            }

            #[test]
            #[cfg(feature = "expensive")]
            fn expensive_examples() {
                use super::SOLUTION;
                $(
                solution_results!($exp_input, $exp_answers);
                )*
            }

            #[test]
            #[ignore]
            fn actual() {
                use super::SOLUTION;
                assert_eq!(&SOLUTION.run_and_print(super::super::YEAR_SOLUTIONS.year).unwrap(), $actual);
            }
        };
    }
}
