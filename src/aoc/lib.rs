//! Support crate for the [Advent of Code solutions](../advent_of_code/index.html).
//!
//! Contains useful abstractions are that are used for one more than one solution.
#![feature(slice_pattern)]
#![feature(assert_matches)]
#![warn(missing_docs)]
#![feature(let_chains)]
#![feature(step_trait)]
#![feature(associated_type_defaults)]
#![feature(impl_trait_in_assoc_type)]
#![warn(clippy::missing_docs_in_private_items)]

pub mod circular_list;
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
        extension::{
            euclid::{AllPoints, BoxInclusive, ConversionExt, ManhattanLen, UnitVectors},
            RangeExt,
        },
        grid::{
            AnyGridPoint, AnyGridPointExt, FromGridStr, Grid, GridBox, GridDefault, GridPoint,
            GridSize, GridSizeExt, GridSpace,
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
        /// Other, miscellaneous error with description.
        #[error("Other error: {0}")]
        Other(Cow<'static, str>),
    }

    /// Result with an [`AocError`].
    pub type AocResult<T> = Result<T, AocError>;
}

/// Collection of general extension traits.
pub mod extension {
    use num::Integer;
    use std::ops::RangeInclusive;

    /// Extension traits for items in the `euclid` geometry crate.
    pub mod euclid {
        use std::borrow::Borrow;

        use euclid::{
            num::{One, Zero},
            Box2D, Box3D, Point2D, Point3D, Size2D, Size3D, Vector2D, Vector3D,
        };
        use itertools::iproduct;
        use num::{NumCast, Signed};

        /// Extension trait for mathematical vectors from that calculates their
        /// [Manhattan length](https://en.wikipedia.org/wiki/Taxicab_geometry).
        pub trait ManhattanLen<T, U> {
            /// Calculates the [Manhattan length](https://en.wikipedia.org/wiki/Taxicab_geometry)
            /// of the vector.
            ///
            /// # Examples
            /// Basic usage:
            /// ```
            /// # use aoc::prelude::*;
            /// use euclid::default::{Vector2D, Vector3D};
            /// assert_eq!(Vector2D::new(0, 0).manhattan_len(), 0);
            /// assert_eq!(Vector2D::new(3, -10).manhattan_len(), 13);
            /// assert_eq!(Vector3D::new(-5, 2, -4).manhattan_len(), 11);
            /// ```
            fn manhattan_len(&self) -> T;
        }
        impl<T: Signed, U> ManhattanLen<T, U> for Vector2D<T, U> {
            fn manhattan_len(&self) -> T {
                self.x.abs() + self.y.abs()
            }
        }
        impl<T: Signed, U> ManhattanLen<T, U> for Vector3D<T, U> {
            fn manhattan_len(&self) -> T {
                self.x.abs() + self.y.abs() + self.z.abs()
            }
        }

        /// Extension trait the provides unit vectors for each 3D axis.
        pub trait UnitVectors {
            /// Returns the positive `x` unit vector.
            ///
            /// # Examples
            /// Basic usage:
            /// ```
            /// # use aoc::prelude::*;
            /// use euclid::default::{Vector2D, Vector3D};
            ///
            /// assert_eq!(Vector2D::<u8>::unit_x(), Vector2D::new(1, 0));
            /// assert_eq!(Vector3D::<u8>::unit_x(), Vector3D::new(1, 0, 0));
            /// ```
            fn unit_x() -> Self;

            /// Returns the positive `y` unit vector.
            ///
            /// # Examples
            /// Basic usage:
            /// ```
            /// # use aoc::prelude::*;
            /// use euclid::default::{Vector2D, Vector3D};
            ///
            /// assert_eq!(Vector2D::<u8>::unit_y(), Vector2D::new(0, 1));
            /// assert_eq!(Vector3D::<u8>::unit_y(), Vector3D::new(0, 1, 0));
            /// ```
            fn unit_y() -> Self;

            /// Returns the positive `z` unit vector.
            ///
            /// # Panics
            /// This will panic if called on a 2D vector.
            ///
            /// # Examples
            /// Basic usage:
            /// ```
            /// # use aoc::prelude::*;
            /// use euclid::default::Vector3D;
            ///
            /// assert_eq!(Vector3D::<u8>::unit_z(), Vector3D::new(0, 0, 1));
            /// ```
            ///
            /// Incorrect usage:
            /// ```should_panic
            /// # use aoc::prelude::*;
            /// use euclid::default::Vector2D;
            ///
            /// let _ = Vector2D::<u8>::unit_z();
            /// ```
            fn unit_z() -> Self;
        }
        impl<T: One + Zero, U> UnitVectors for Vector2D<T, U> {
            fn unit_x() -> Self {
                Self::new(T::one(), T::zero())
            }

            fn unit_y() -> Self {
                Self::new(T::zero(), T::one())
            }

            fn unit_z() -> Self {
                panic!("a 2D vector has no z coordinate")
            }
        }
        impl<T: One + Zero, U> UnitVectors for Vector3D<T, U> {
            fn unit_x() -> Self {
                Self::new(T::one(), T::zero(), T::zero())
            }

            fn unit_y() -> Self {
                Self::new(T::zero(), T::one(), T::zero())
            }

            fn unit_z() -> Self {
                Self::new(T::zero(), T::zero(), T::one())
            }
        }

        /// Extension trait to provide additional component type conversions.
        pub trait ConversionExt {
            /// The item we are extending with its generic component type.
            type Item<S>;

            /// Converts the item components to [`isize`].
            ///  
            /// # Panics
            /// This will panic if any of the components cannot be converted.
            fn to_isize(self) -> Self::Item<isize>;

            /// Converts the item components to [`u64`].
            ///
            /// # Panics
            /// This will panic if any of the components cannot be converted.
            fn to_u64(self) -> Self::Item<u64>;
        }

        /// Implements [`ConversionExt`] for a particular `euclid` item.
        ///
        /// The item must have the `try_cast` method.
        macro_rules! impl_euclid_conversions {
            ($T:ident) => {
                impl<T: NumCast + Copy, U> ConversionExt for $T<T, U> {
                    type Item<S> = $T<S, U>;

                    fn to_isize(self) -> Self::Item<isize> {
                        self.try_cast().unwrap()
                    }

                    fn to_u64(self) -> Self::Item<u64> {
                        self.try_cast().unwrap()
                    }
                }
            };
        }

        impl_euclid_conversions!(Point2D);
        impl_euclid_conversions!(Point3D);
        impl_euclid_conversions!(Size2D);
        impl_euclid_conversions!(Size3D);
        impl_euclid_conversions!(Vector2D);
        impl_euclid_conversions!(Vector3D);
        impl_euclid_conversions!(Box2D);
        impl_euclid_conversions!(Box3D);

        /// Extension trait to define boxes using inclusive points.
        ///
        /// The compensates for the fact that the `max` point of a `euclid`
        /// box item is exclusive.
        pub trait BoxInclusive {
            /// The point type that defines the box.
            type Point;

            /// Returns a new box with `min` and `max` points that are both contained in the box.
            ///
            /// # Examples
            /// Basic usage:
            /// ```
            /// # use aoc::prelude::*;
            /// use euclid::default::{Box2D, Box3D, Point2D, Point3D, Size2D, Size3D, Vector2D, Vector3D};
            ///
            /// let min = Point2D::new(1, 2);
            /// let max = Point2D::new(6, 7);
            /// let bounds = Box2D::new_inclusive(min, max);
            /// assert!(bounds.contains(min));
            /// assert!(bounds.contains(max));
            /// assert!(!bounds.contains(min - Vector2D::unit_x()));
            /// assert!(!bounds.contains(max + Vector2D::unit_x()));
            /// assert_eq!(bounds.size(), Size2D::new(6, 6));
            ///
            /// let min = Point3D::new(1, 2, 3);
            /// let max = Point3D::new(6, 7, 8);
            /// let bounds = Box3D::new_inclusive(min, max);
            /// assert!(bounds.contains(min));
            /// assert!(bounds.contains(max));
            /// assert!(!bounds.contains(min - Vector3D::unit_y()));
            /// assert!(!bounds.contains(max + Vector3D::unit_y()));
            /// assert_eq!(bounds.size(), Size3D::new(6, 6, 6));
            /// ```
            fn new_inclusive(min: Self::Point, max: Self::Point) -> Self;

            /// Returns the smallest box containing **all** of the provided points.
            ///
            /// [`Box2D::from_points`] and [`Box3D::from_points`] will create
            /// boxes that actually exclude one or more points on the back edges of
            /// the box.
            /// I thought this may be a bug, but it turns out that this is the
            /// [intended behavior](https://github.com/servo/euclid/issues/519).
            ///
            /// # Examples
            /// Basic usage:
            /// ```
            /// # use aoc::prelude::*;
            /// use euclid::default::{Box2D, Box3D, Point2D, Point3D};
            ///
            /// let points = [(1, 2), (2, 0), (4, 9), (6, 7), (7, 8)].map(Point2D::from);
            /// assert_eq!(
            ///     Box2D::from_points_inclusive(points),
            ///     Box2D::new_inclusive((1, 0).into(), (7, 9).into())
            /// );
            ///
            /// let points = [(1, 2, 8), (2, 0, 4), (4, 9, -8), (6, 7, -5), (7, 8, 3)].map(Point3D::from);
            /// assert_eq!(
            ///     Box3D::from_points_inclusive(points),
            ///     Box3D::new_inclusive((1, 0, -8).into(), (7, 9, 8).into())
            /// );
            /// ```
            fn from_points_inclusive<I>(points: I) -> Self
            where
                I: IntoIterator,
                I::Item: Borrow<Self::Point>;
        }
        impl<T: Copy + std::ops::Add<Output = T> + One + Zero + PartialOrd, U> BoxInclusive
            for Box2D<T, U>
        {
            type Point = Point2D<T, U>;

            fn new_inclusive(min: Self::Point, max: Self::Point) -> Self {
                Self::new(min, max + Vector2D::new(T::one(), T::one()))
            }

            fn from_points_inclusive<I>(points: I) -> Self
            where
                I: IntoIterator,
                I::Item: Borrow<Self::Point>,
            {
                {
                    let bounds = Self::from_points(points);
                    Self::new_inclusive(bounds.min, bounds.max)
                }
            }
        }
        impl<T: Copy + std::ops::Add<Output = T> + One + Zero + PartialOrd, U> BoxInclusive
            for Box3D<T, U>
        {
            type Point = Point3D<T, U>;

            fn new_inclusive(min: Self::Point, max: Self::Point) -> Self {
                Self::new(min, max + Vector3D::new(T::one(), T::one(), T::one()))
            }

            fn from_points_inclusive<I>(points: I) -> Self
            where
                I: IntoIterator,
                I::Item: Borrow<Self::Point>,
            {
                {
                    let bounds = Self::from_points(points);
                    Self::new_inclusive(bounds.min, bounds.max)
                }
            }
        }

        /// Extension trait for iterating over all the points contained in an
        /// appropriate `euclid` item.
        pub trait AllPoints {
            /// The type of the point to be contained in the item.
            type Point;

            /// The iterator type returned from [`AllPoints::all_points`].
            ///
            /// This is needed due to a
            /// [limitation of RPITIT](https://users.rust-lang.org/t/fully-owned-iterator-causing-lifetime-problems/107677).
            type AllPointsIterator: Iterator<Item = Self::Point>;

            /// Returns an [`Iterator`] over all points contained in the item in row-major order.
            ///
            /// # Examples
            /// Basic usage:
            /// ```
            /// # use aoc::prelude::*;
            /// use itertools::Itertools;
            /// use euclid::default::{Box2D, Box3D, Point2D, Point3D, Size2D, Size3D};
            ///
            /// assert_eq!(
            ///     Size2D::new(2, 3).all_points().collect_vec(),
            ///     vec![
            ///         Point2D::new(0, 0),
            ///         Point2D::new(1, 0),
            ///         Point2D::new(0, 1),
            ///         Point2D::new(1, 1),
            ///         Point2D::new(0, 2),
            ///         Point2D::new(1, 2),
            ///     ],
            /// );
            /// assert_eq!(
            ///     Size3D::new(2, 2, 3).all_points().collect_vec(),
            ///     vec![
            ///         Point3D::new(0, 0, 0),
            ///         Point3D::new(1, 0, 0),
            ///         Point3D::new(0, 1, 0),
            ///         Point3D::new(1, 1, 0),
            ///         Point3D::new(0, 0, 1),
            ///         Point3D::new(1, 0, 1),
            ///         Point3D::new(0, 1, 1),
            ///         Point3D::new(1, 1, 1),
            ///         Point3D::new(0, 0, 2),
            ///         Point3D::new(1, 0, 2),
            ///         Point3D::new(0, 1, 2),
            ///         Point3D::new(1, 1, 2),
            ///     ],
            /// );
            /// assert_eq!(
            ///     Box2D::from_origin_and_size(Point2D::new(3, 4), Size2D::new(2, 3)).all_points().collect_vec(),
            ///     vec![
            ///         Point2D::new(3, 4),
            ///         Point2D::new(4, 4),
            ///         Point2D::new(3, 5),
            ///         Point2D::new(4, 5),
            ///         Point2D::new(3, 6),
            ///         Point2D::new(4, 6),
            ///     ],
            /// );
            /// assert_eq!(
            ///     Box3D::from_origin_and_size(Point3D::new(3, 4, 5), Size3D::new(2, 2, 3)).all_points().collect_vec(),
            ///     vec![
            ///         Point3D::new(3, 4, 5),
            ///         Point3D::new(4, 4, 5),
            ///         Point3D::new(3, 5, 5),
            ///         Point3D::new(4, 5, 5),
            ///         Point3D::new(3, 4, 6),
            ///         Point3D::new(4, 4, 6),
            ///         Point3D::new(3, 5, 6),
            ///         Point3D::new(4, 5, 6),
            ///         Point3D::new(3, 4, 7),
            ///         Point3D::new(4, 4, 7),
            ///         Point3D::new(3, 5, 7),
            ///         Point3D::new(4, 5, 7),
            ///     ],
            /// );
            /// ```
            fn all_points(&self) -> Self::AllPointsIterator;
        }
        impl<T: Copy + std::iter::Step + euclid::num::Zero, U> AllPoints for Size2D<T, U> {
            type Point = Point2D<T, U>;
            type AllPointsIterator = impl Iterator<Item = Self::Point>;

            fn all_points(&self) -> Self::AllPointsIterator {
                iproduct!(T::zero()..self.height, T::zero()..self.width)
                    .map(|(y, x)| Self::Point::new(x, y))
            }
        }
        impl<T: Copy + std::iter::Step + euclid::num::Zero, U> AllPoints for Size3D<T, U> {
            type Point = Point3D<T, U>;
            type AllPointsIterator = impl Iterator<Item = Self::Point>;

            fn all_points(&self) -> Self::AllPointsIterator {
                iproduct!(
                    T::zero()..self.depth,
                    T::zero()..self.height,
                    T::zero()..self.width
                )
                .map(|(z, y, x)| Self::Point::new(x, y, z))
            }
        }
        impl<T: Copy + std::iter::Step, U> AllPoints for Box2D<T, U> {
            type Point = Point2D<T, U>;
            type AllPointsIterator = impl Iterator<Item = Self::Point>;

            fn all_points(&self) -> Self::AllPointsIterator {
                iproduct!(self.min.y..self.max.y, self.min.x..self.max.x)
                    .map(|(y, x)| Self::Point::new(x, y))
            }
        }
        impl<T: Copy + std::iter::Step, U> AllPoints for Box3D<T, U> {
            type Point = Point3D<T, U>;
            type AllPointsIterator = impl Iterator<Item = Self::Point>;

            fn all_points(&self) -> Self::AllPointsIterator {
                iproduct!(
                    self.min.z..self.max.z,
                    self.min.y..self.max.y,
                    self.min.x..self.max.x
                )
                .map(|(z, y, x)| Self::Point::new(x, y, z))
            }
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
