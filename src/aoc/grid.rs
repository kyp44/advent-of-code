//! 2D grids of values.
//!
//! Contains the main [`Grid`] struct, associated traits, and some useful
//! grid element types.
use super::prelude::*;
use cgmath::{EuclideanSpace, Point2, Vector2};
use core::slice::SlicePattern;
use derive_more::{Add, AddAssign, Deref, From, Into, Not, Sub, SubAssign};
use itertools::{iproduct, process_results, Itertools};
use num::FromPrimitive;
use std::{cmp::Eq, collections::HashSet, fmt, hash::Hash, str::FromStr};

/// A point location in a [`Grid`] that should be within the bounds of the grid.
pub type GridPoint = Point2<usize>;
/// The size of a [`Grid`].
pub type GridSize = Vector2<usize>;
/// A point location in any [`Grid`] regardless of its bounds.
pub type AnyGridPoint = Point2<isize>;

// TODO: Extension for GridPoint with moved Grid::all_neighbor_points.

/// Extension trait for [`GridSize`].
pub trait GridSizeExt {
    /// Returns an [`Iterator`] over all points in a grid of this size in row-major order.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let size = GridSize::new(2, 3);
    /// let points = size.all_points().collect::<Vec<_>>();
    ///
    /// assert_eq!(points, vec![
    ///     GridPoint::new(0, 0),
    ///     GridPoint::new(1, 0),
    ///     GridPoint::new(0, 1),
    ///     GridPoint::new(1, 1),
    ///     GridPoint::new(0, 2),
    ///     GridPoint::new(1, 2),
    /// ]);
    /// ```
    fn all_points(&self) -> Box<dyn Iterator<Item = GridPoint>>;

    /// Validates that a size if valid for a grid.
    ///
    /// Valid sizes are those that are nonzero in both dimensions.
    ///
    /// # Panics
    /// This will panic if the size is invalid.
    fn validate(self) -> Self;
}
impl GridSizeExt for GridSize {
    fn all_points(&self) -> Box<dyn Iterator<Item = GridPoint>> {
        Box::new(iproduct!(0..self.y, 0..self.x).map(|(y, x)| GridPoint::new(x, y)))
    }

    fn validate(self) -> Self {
        if self.x == 0 || self.y == 0 {
            panic!("A GridSize of (${}, ${}) is invalid", self.x, self.y);
        }
        self
    }
}

/// A 2D grid of values.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Grid<T> {
    /// The size of the grid.
    size: GridSize,
    /// The actual grid data of fixed size.
    ///
    /// This is an array of arrays in which each inner array is a row and is
    /// guaranteed to have the same length, which is of course the width of
    /// the grid.
    data: Box<[Box<[T]>]>,
}
impl<T: Default + Clone> Grid<T> {
    /// Creates a default grid of a particular `size` with default values.
    ///
    /// # Panics
    /// This will panic if the `size` is invalid, that is it contains zero in either dimension.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// # use aoc::grid::Digit;
    /// let grid = Grid::<Digit>::default(GridSize::new(3, 3));
    ///
    /// assert_eq!(format!("{grid:?}"), "000\n000\n000\n");
    /// ```
    pub fn default(size: GridSize) -> Self {
        Self {
            size: size.validate(),
            data: vec![vec![T::default(); size.x].into_boxed_slice(); size.y].into_boxed_slice(),
        }
    }
}
impl<T> Grid<T> {
    /// Creates a grid from raw data.
    ///
    /// The raw data should be a [`Vec`] for rows, with each row being
    /// itself a [`Vec`] of the same length. Returns an [`AocError::InvalidInput`]
    /// if either the passed data is empty, or if there are rows with different
    /// lengths.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// # use aoc::grid::Digit;
    /// let grid = Grid::<Digit>::from_data(vec![
    ///     vec![0.into(), 1.into()],
    ///     vec![2.into(), 3.into()],
    ///     vec![4.into(), 5.into()],
    /// ]).unwrap();
    /// assert_eq!(format!("{grid:?}"), "01\n23\n45\n");
    /// ```
    ///
    /// Invalid usage:
    /// ```
    /// # #![feature(assert_matches)]
    /// # use std::assert_matches::assert_matches;
    /// # use aoc::prelude::*;
    /// # use aoc::grid::Digit;
    /// assert_matches!(Grid::<u8>::from_data(vec![]), Err(AocError::InvalidInput(_)));
    ///
    /// let result = Grid::from_data(vec![
    ///     vec![1, 2, 3],
    ///     vec![4, 5],
    ///     vec![6],
    /// ]);
    /// assert_matches!(result, Err(AocError::InvalidInput(_)));
    /// ```
    pub fn from_data(data: Vec<Vec<T>>) -> AocResult<Self> {
        // Verify that we have at least one row
        let height = data.len();
        if height < 1 {
            return Err(AocError::InvalidInput("The grid has no content!".into()));
        }

        // Verify that all the row widths are the same
        let width = data[0].len();
        let data = process_results(
            data.into_iter().map(|row| {
                if row.len() != width {
                    Err(AocError::InvalidInput(
                        format!(
                            "Grid row has a length of {} instead of the expected {}",
                            row.len(),
                            width
                        )
                        .into(),
                    ))
                } else {
                    Ok(row.into_boxed_slice())
                }
            }),
            |iter| iter.collect(),
        )?;

        Ok(Self {
            size: GridSize::new(width, height),
            data,
        })
    }

    /// Returns the size the grid.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let grid = Grid::<u8>::from_data(vec![vec![1, 2], vec![3, 4], vec![5, 6]]).unwrap();
    ///
    /// assert_eq!(*grid.size(), GridSize::new(2, 3));
    /// ```
    pub fn size(&self) -> &GridSize {
        &self.size
    }

    /// Gets a reference to the element at a location.
    ///
    /// # Panics
    /// This will panic if the location is out of the bounds of the grid based on
    /// its size.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let grid = Grid::<u8>::from_data(vec![vec![1, 2], vec![3, 4], vec![5, 6]]).unwrap();
    ///
    /// assert_eq!(*grid.get(&GridPoint::new(0, 1)), 3);
    /// assert_eq!(*grid.get(&GridPoint::new(1, 1)), 4);
    /// assert_eq!(*grid.get(&GridPoint::new(0, 0)), 1);
    /// ```
    pub fn get(&self, point: &GridPoint) -> &T {
        &self.data[point.y][point.x]
    }

    /// Sets element at a location.
    ///
    /// # Panics
    /// This will panic if the location is out of the bounds of the grid based on
    /// its size.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let mut grid = Grid::<u8>::from_data(vec![vec![1, 2], vec![3, 4], vec![5, 6]]).unwrap();
    /// let point = GridPoint::new(1, 1);
    ///
    /// assert_eq!(*grid.get(&point), 4);
    ///
    /// grid.set(&point, 21);
    /// assert_eq!(*grid.get(&point), 21);
    /// ```
    pub fn set(&mut self, point: &GridPoint, value: T) {
        *self.element_at(point) = value;
    }

    /// Gets a mutable reference to an element.
    ///
    /// # Panics
    /// This will panic if the location is out of the bounds of the grid based on
    /// its size.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let mut grid = Grid::<u8>::from_data(vec![vec![1, 2], vec![3, 4], vec![5, 6]]).unwrap();
    /// let point = GridPoint::new(1, 1);
    ///
    /// assert_eq!(*grid.element_at(&point), 4);
    /// *grid.element_at(&point) =  21;
    /// assert_eq!(*grid.element_at(&point), 21);
    /// ```
    pub fn element_at(&mut self, point: &GridPoint) -> &mut T {
        &mut self.data[point.y][point.x]
    }

    /// Verifies that any grid point is in the bounds of the grid and converts it if so.
    ///
    /// If the signed `point` is out bounds, then `None` will be returned.
    /// If it is in bounds then the corresponding unsigned point will be returned.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let grid = Grid::<bool>::default(GridSize::new(2, 3));
    ///
    /// assert_eq!(grid.valid_point(&AnyGridPoint::new(0, 1)), Some(GridPoint::new(0, 1)));
    /// assert_eq!(grid.valid_point(&AnyGridPoint::new(1, 2)), Some(GridPoint::new(1, 2)));
    /// assert_eq!(grid.valid_point(&AnyGridPoint::new(0, -1)), None);
    /// assert_eq!(grid.valid_point(&AnyGridPoint::new(2, 0)), None);
    /// ```
    pub fn bounded_point(&self, point: &AnyGridPoint) -> Option<GridPoint> {
        if point.x >= 0 && point.y >= 0 {
            let point = GridPoint::try_point_from(*point).unwrap();
            let size = self.size();
            if point.x < size.x && point.y < size.y {
                Some(point)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns an [`Iterator`] over all valid grid points in row-major order.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let grid = Grid::<u8>::from_data(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap();
    /// let points = vec![
    ///     GridPoint::new(0, 0),
    ///     GridPoint::new(1, 0),
    ///     GridPoint::new(2, 0),
    ///     GridPoint::new(0, 1),
    ///     GridPoint::new(1, 1),
    ///     GridPoint::new(2, 1),
    /// ];
    ///
    /// assert_eq!(grid.all_points().collect::<Vec<_>>(), points);
    /// ```
    pub fn all_points(&self) -> impl Iterator<Item = GridPoint> {
        self.size().all_points()
    }

    /// Returns an [`Iterator`] over all grid values in row-major order.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let grid = Grid::<u8>::from_data(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap();
    ///
    /// assert_eq!(
    ///     grid.all_values().copied().collect::<Vec<_>>(),
    ///     vec![1, 2, 3, 4, 5, 6]
    /// );
    /// ```
    pub fn all_values(&self) -> impl Iterator<Item = &T> {
        Box::new(self.all_points().map(|p| self.get(&p)))
    }

    /// Returns an [`Iterator`] over the values in a `row`.
    ///
    /// # Panics
    /// This will panic if the `row` has a value that is out of bounds for the size of the grid.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let grid = Grid::<u8>::from_data(vec![vec![1, 2], vec![3, 4], vec![5, 6]]).unwrap();
    ///
    /// assert_eq!(grid.row_iter(1).copied().collect::<Vec<_>>(), vec![3, 4]);
    /// ```
    pub fn row_iter(&self, row: usize) -> impl Iterator<Item = &T> {
        self.data[row].iter()
    }

    /// Returns an [`Iterator`] over the values in a `column`.
    ///
    /// # Panics
    /// This will panic if the `column` has a value that is out of bounds for the size of the grid.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let grid = Grid::<u8>::from_data(vec![vec![1, 4], vec![2, 5], vec![3, 6]]).unwrap();
    ///
    /// assert_eq!(grid.column_iter(1).copied().collect::<Vec<_>>(), vec![4, 5, 6]);
    /// ```
    pub fn column_iter(&self, column: usize) -> impl Iterator<Item = &T> {
        (0..self.size.y).map(move |y| &self.data[y][column])
    }

    /// Returns an [`Iterator`] over the rows as slices.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let grid = Grid::<u8>::from_data(vec![vec![1, 4], vec![2, 5], vec![3, 6]]).unwrap();
    /// let mut iter = grid.rows_iter();
    ///
    /// assert_eq!(iter.next().unwrap(), &[1, 4]);
    /// assert_eq!(iter.next().unwrap(), &[2, 5]);
    /// assert_eq!(iter.next().unwrap(), &[3, 6]);
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn rows_iter(&self) -> impl Iterator<Item = &[T]> {
        self.data.iter().map(|row| row.as_slice())
    }

    /// Returns an [`Iterator`] over all the neighboring points around a `point`
    /// in row-major order.
    ///
    /// Note that this is independent of any particular [`Grid`] instance.
    /// May optionally include the four diagonal neighbor points as well as this
    /// `point` itself. These options will dictate the length of the [`Iterator`].
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    ///
    /// assert_eq!(Grid::all_neighbor_points(AnyGridPoint::new()))
    /// ```
    pub fn all_neighbor_points(
        point: AnyGridPoint,
        include_diagonals: bool,
        include_self: bool,
    ) -> impl Iterator<Item = AnyGridPoint> {
        iproduct!(-1isize..=1, -1isize..=1).filter_map(move |(dy, dx)| {
            let point = point + Vector2::new(dx, dy);
            if dx == 0 && dy == 0 {
                if include_self {
                    Some(point)
                } else {
                    None
                }
            } else if !include_diagonals && (dx + dy).abs() != 1 {
                None
            } else {
                Some(point)
            }
        })
    }

    /// Returns an [`Iterator`] over all the neighboring points around a `point`
    /// in row-major order regardless of whether the neighboring points are
    /// actually within the bounds of the grid or not.
    ///
    /// The length of the [`Iterator`] will depend on the location of the `point`.
    /// May optionally include the (up to) four diagonal neighbor points as well
    /// as this `point` itself.
    ///
    /// # Examples
    /// Basic ussage:
    /// ```
    /// # use aoc::prelude::*;
    /// let grid = Grid::<u8>::from_data(vec![vec![1, 2, 4], vec![5, 6, 7], vec![3, 6]]).unwrap();
    ///
    /// dasdas
    /// ```
    pub fn neighbor_points<'a>(
        &'a self,
        point: &GridPoint,
        include_diagonals: bool,
        include_self: bool,
    ) -> impl Iterator<Item = GridPoint> + 'a {
        Self::all_neighbor_points(
            AnyGridPoint::try_point_from(*point).unwrap(),
            include_diagonals,
            include_self,
        )
        .filter_map(|p| self.bounded_point(&p))
    }

    pub fn sub_grid(&self, point: &GridPoint, size: GridSize) -> Self
    where
        T: Default + Clone,
    {
        let point = point.to_vec();
        let mut out = Self::default(size);
        for out_point in out.all_points() {
            out.set(&out_point, self.get(&(&out_point + point)).clone());
        }
        out
    }
}
// Additional methods for grids with boolean-like elements.
impl<T: From<bool> + Default + Clone> Grid<T> {
    pub fn from_coordinates(points: impl Iterator<Item = AnyGridPoint> + Clone) -> Self {
        let x_range = points.clone().map(|p| p.x).range().unwrap_or(0..=0);
        let y_range = points.clone().map(|p| p.y).range().unwrap_or(0..=0);
        let size = GridSize::new(
            x_range.len().try_into().unwrap(),
            y_range.len().try_into().unwrap(),
        );
        let mut grid = Self::default(size);

        for point in points.map(|p| {
            GridPoint::new(
                (p.x - x_range.start()).try_into().unwrap(),
                (p.y - y_range.start()).try_into().unwrap(),
            )
        }) {
            grid.set(&point, true.into());
        }
        grid
    }
}
impl<T: Into<bool> + Clone> Grid<T> {
    pub fn as_coordinates(&self) -> HashSet<GridPoint> {
        self.all_points()
            .filter(|p| Into::<bool>::into(self.get(p).clone()))
            .collect()
    }
}

/// Parsing a grid from a grid of characters with each row on a separate line.
///
/// This can be done for element types that can be fallibly converted from characters.
impl<T: TryFrom<char>> FromStr for Grid<T> {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = s
            .lines()
            .map(|line| {
                line.chars()
                    .map(|c| {
                        T::try_from(c).map_err(|_| {
                            AocError::InvalidInput(format!("Invalid character found: '{c}'").into())
                        })
                    })
                    .collect()
            })
            .collect::<Result<_, _>>()?;
        Self::from_data(data)
    }
}
/// Debug display for grid whose elements implement [`Debug`].
impl<T: fmt::Debug> fmt::Debug for Grid<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = self.size();
        writeln!(
            f,
            "{}",
            (0..size.y)
                .map(|y| (0..size.x)
                    .map(|x| format!("{:?}", self.get(&GridPoint::new(x, y))))
                    .collect::<String>())
                .join("\n")
        )
    }
}

/// Create an object from a [`GridSize`].
pub trait GridDefault<T: Default + Clone>: From<Grid<T>> {
    /// Returns a default object from particular [`GridSize`].
    fn default(size: GridSize) -> Self {
        Grid::default(size).into()
    }
}
/// Parse objects that can be created from a grid from a grid of characters.
///
/// Note that we cannot just blanket implement [`FromStr`] due to the orphan rule.
pub trait FromGridStr<T>: Sized {
    /// The error type if the conversion fails.
    type Err;

    /// Creates an object from a grid of characters.
    fn from_grid_str(s: &str) -> Result<Self, Self::Err>;
}
impl<T: TryFrom<char>, O: From<Grid<T>>> FromGridStr<T> for O {
    type Err = AocError;

    fn from_grid_str(s: &str) -> Result<Self, Self::Err> {
        Ok(<Grid<T> as FromStr>::from_str(s)?.into())
    }
}

/// Standard boolean grid element where '.' is false and '#' is true.
#[derive(Deref, From, Into, Not, Default, Clone, Copy)]
pub struct StdBool(bool);
impl TryFrom<char> for StdBool {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '#' => Ok(true.into()),
            '.' => Ok(false.into()),
            _ => Err(()),
        }
    }
}
impl fmt::Debug for StdBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", if **self { '#' } else { '.' })
    }
}

/// Grid of numbers parsed from digit characters.
#[derive(
    Deref,
    From,
    Into,
    Default,
    Clone,
    Copy,
    Add,
    Sub,
    AddAssign,
    SubAssign,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub struct Digit(u8);
impl TryFrom<char> for Digit {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        value
            .to_digit(10)
            .map(|d| Self(d.try_into().unwrap()))
            .ok_or(())
    }
}
impl fmt::Debug for Digit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", **self)
    }
}
impl FromPrimitive for Digit {
    fn from_i64(n: i64) -> Option<Self> {
        u8::try_from(n).ok().map(|n| n.into())
    }

    fn from_u64(n: u64) -> Option<Self> {
        u8::try_from(n).ok().map(|n| n.into())
    }
}
