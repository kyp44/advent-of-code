//! 2D grids of values.
//!
//! Contains the main [`Grid`] struct, associated traits, and some useful
//! grid element types.

use super::prelude::*;
use derive_more::{Add, AddAssign, Deref, From, Into, Not, Sub, SubAssign};
use euclid::{Box2D, Point2D, Size2D, Vector2D};
use itertools::iproduct;
use num::FromPrimitive;
use petgraph::{graph::NodeIndex, stable_graph::IndexType, EdgeType, Graph};
use std::{cmp::Eq, collections::HashSet, fmt, hash::Hash, marker::PhantomData, str::FromStr};

/// A grid coordinate system in which the origin is the in upper left of the grid
/// and increasing `y` moves down in the grid.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridSpace;

/// The size of a [`Grid`].
///
/// Sizes in which either element being zero is not valid.
pub type GridSize<U = GridSpace> = Size2D<usize, U>;

/// A point location in a [`Grid`] that should be within the bounds of the grid.
///
/// Refer to [`GridSpace`] for the coordinate system.
pub type GridPoint<U = GridSpace> = Point2D<usize, U>;

/// A point location in any [`Grid`] regardless of its bounds.
///
/// Refer to [`GridSpace`] for the coordinate system.
pub type AnyGridPoint<U = GridSpace> = Point2D<isize, U>;

/// A box within a [`Grid`] that defines a sub-grid.
///
/// Refer to [`GridSpace`] for the coordinate system.
pub type GridBox<U = GridSpace> = Box2D<usize, U>;

/// Extension trait for [`GridSize`].
pub trait GridSizeExt<U>: Sized {
    /// Returns whether the size is valid, that is nonzero in both dimensions.
    fn is_valid(&self) -> bool;

    /// Panics if the size is not valid, see [`GridSizeExt::is_valid`].
    fn validate(&self) {
        if !self.is_valid() {
            panic!("grid size is invalid");
        }
    }

    /// Creates a size and verifies that it is valid, see [`GridSizeExt::is_valid`].
    fn new_valid(width: usize, height: usize) -> Option<Self>;
}
impl<U> GridSizeExt<U> for GridSize<U> {
    fn new_valid(width: usize, height: usize) -> Option<Self> {
        let size = Self::new(width, height);
        (size.is_valid()).then_some(size)
    }

    fn is_valid(&self) -> bool {
        !self.is_empty()
    }
}

/// Extension trait for [`AnyGridPoint`].
pub trait AnyGridPointExt<U> {
    /// The iterator type returned from [`AnyGridPointExt::all_neighbor_points`].
    ///
    /// This is needed due to a
    /// [limitation of RPITIT](https://users.rust-lang.org/t/fully-owned-iterator-causing-lifetime-problems/107677).
    type NeighborPoints: Iterator<Item = AnyGridPoint<U>>;

    /// Converts an infinitely repeating grid point into the actual addressable point
    /// for a given grid size.
    ///
    /// This is effectively just a modulo operation in both dimensions using the `size`
    /// values as the modulo.
    ///
    /// # Panics
    /// This will panic if any of the conversions fail from signed to unsigned types
    /// and vice versa.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let size = GridSize::<GridSpace>::new(5, 4);
    /// assert_eq!(
    ///     AnyGridPoint::new(1, 2).wrapped_grid_point(&size),
    ///     GridPoint::new(1, 2),
    /// );
    /// assert_eq!(
    ///     AnyGridPoint::new(-1, -2).wrapped_grid_point(&size),
    ///     GridPoint::new(4, 2),
    /// );
    /// assert_eq!(
    ///     AnyGridPoint::new(-789, 27615).wrapped_grid_point(&size),
    ///     GridPoint::new(1, 3),
    /// );
    /// ```
    fn wrapped_grid_point(&self, size: &GridSize<U>) -> GridPoint<U>;

    /// Returns an [`Iterator`] over all the neighboring points around a `point`
    /// in row-major order.
    ///
    /// The set of points may optionally include the four diagonal neighbor points
    /// as well as this `point` itself. These options will dictate the length of
    /// the [`Iterator`].
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use itertools::Itertools;
    ///
    /// assert_eq!(
    ///     AnyGridPoint::<GridSpace>::new(0, 0)
    ///         .all_neighbor_points(true, true)
    ///         .collect_vec(),
    ///     vec![
    ///         AnyGridPoint::new(-1, -1),
    ///         AnyGridPoint::new(0, -1),
    ///         AnyGridPoint::new(1, -1),
    ///         AnyGridPoint::new(-1, 0),
    ///         AnyGridPoint::new(0, 0),
    ///         AnyGridPoint::new(1, 0),
    ///         AnyGridPoint::new(-1, 1),
    ///         AnyGridPoint::new(0, 1),
    ///         AnyGridPoint::new(1, 1),
    ///     ],
    /// );
    /// assert_eq!(
    ///     AnyGridPoint::<GridSpace>::new(-4, -2)
    ///         .all_neighbor_points(true, false)
    ///         .collect_vec(),
    ///     vec![
    ///         AnyGridPoint::new(-5, -3),
    ///         AnyGridPoint::new(-4, -3),
    ///         AnyGridPoint::new(-3, -3),
    ///         AnyGridPoint::new(-5, -2),
    ///         AnyGridPoint::new(-3, -2),
    ///         AnyGridPoint::new(-5, -1),
    ///         AnyGridPoint::new(-4, -1),
    ///         AnyGridPoint::new(-3, -1),
    ///     ],
    /// );
    /// assert_eq!(
    ///     AnyGridPoint::<GridSpace>::new(5, 6)
    ///         .all_neighbor_points(false, false)
    ///         .collect_vec(),
    ///     vec![
    ///         AnyGridPoint::new(5, 5),
    ///         AnyGridPoint::new(4, 6),
    ///         AnyGridPoint::new(6, 6),
    ///         AnyGridPoint::new(5, 7),
    ///     ],
    /// );
    /// ```
    fn all_neighbor_points(
        &self,
        include_diagonals: bool,
        include_self: bool,
    ) -> Self::NeighborPoints;
}
impl<U> AnyGridPointExt<U> for AnyGridPoint<U> {
    type NeighborPoints = impl Iterator<Item = Self>;

    fn wrapped_grid_point(&self, size: &GridSize<U>) -> GridPoint<U> {
        self.rem_euclid(&size.to_isize()).to_usize()
    }

    fn all_neighbor_points(
        &self,
        include_diagonals: bool,
        include_self: bool,
    ) -> Self::NeighborPoints {
        let point = *self;

        iproduct!(-1isize..=1, -1isize..=1).filter_map(move |(dy, dx)| {
            let vector = Vector2D::new(dx, dy);
            match vector.manhattan_len() {
                0 => include_self,
                2 => include_diagonals,
                _ => true,
            }
            .then(|| point + vector)
        })
    }
}

/// A 2D grid of values.
///
/// The values are addressed by a [`GridPoint`].
/// Conceptually, the origin point (0, 0) is in the upper-left corner of the grid
/// with increasing `x` moving to the right, and increasing `y` moving down.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Grid<T, U = GridSpace>(grid::Grid<T>, PhantomData<U>);
impl<T, U> Grid<T, U> {
    /// Creates a grid from raw data.
    ///
    /// The raw data should be a [`Vec`] of rows, with each row being
    /// itself a [`Vec`] of the same length. Returns an [`AocError::Other`]
    /// if either the passed data is empty, or if there are rows with different
    /// lengths.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// // This is a grid with the following values:
    /// // 0 1
    /// // 2 3
    /// // 4 5
    /// let grid = Grid::<u8>::from_data(vec![vec![0, 1], vec![2, 3], vec![4, 5]]).unwrap();
    ///
    /// // This is a grid with a single element.
    /// let grid = Grid::<u8>::from_data(vec![vec![5]]).unwrap();
    /// ```
    ///
    /// Invalid usage:
    /// ```
    /// # #![feature(assert_matches)]
    /// # use std::assert_matches::assert_matches;
    /// # use aoc::prelude::*;
    /// assert_matches!(Grid::<u8>::from_data(vec![]), Err(AocError::Other(_)));
    ///
    /// let result = Grid::<_, GridSpace>::from_data(vec![vec![1, 2, 3], vec![4, 5], vec![6]]);
    /// assert_matches!(result, Err(AocError::Other(_)));
    /// ```
    pub fn from_data(data: Vec<Vec<T>>) -> AocResult<Self> {
        // Verify that the data has content
        let size = Self::data_size(&data)?;

        // Verify that all the row widths are the same
        for row in data.iter() {
            if row.len() != size.width {
                return Err(AocError::Other(
                    format!(
                        "Grid row has a length of {} instead of the expected {}",
                        row.len(),
                        size.width
                    )
                    .into(),
                ));
            }
        }

        Ok(Self(data.into(), Default::default()))
    }

    /// Returns the size the grid.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let grid = Grid::<u8>::from_data(vec![vec![1, 2], vec![3, 4], vec![5, 6]]).unwrap();
    ///
    /// assert_eq!(grid.size(), GridSize::new(2, 3));
    /// ```
    pub fn size(&self) -> GridSize<U> {
        let size = self.0.size();

        GridSize::new(size.1, size.0)
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
    pub fn get(&self, point: &GridPoint<U>) -> &T {
        self.0
            .get(point.y, point.x)
            .expect("Grid point is out of bounds")
    }

    /// Gets a reference to the element at any location, if the location is within
    /// the bounds of the grid.
    ///
    /// # Panics
    /// This will panic if the `point` is within the grid bounds but cannot be converted to a
    /// [`GridPoint`].
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # #![feature(assert_matches)]
    /// # use std::assert_matches::assert_matches;
    /// # use aoc::prelude::*;
    /// let grid = Grid::<u8>::from_data(vec![vec![1, 2], vec![3, 4], vec![5, 6]]).unwrap();
    ///
    /// assert_matches!(grid.get_any(&AnyGridPoint::new(0, 1)), Some(&3));
    /// assert_matches!(grid.get_any(&AnyGridPoint::new(-1, 1)), None);
    /// assert_matches!(grid.get_any(&AnyGridPoint::new(1, 2)), Some(&6));
    /// assert_matches!(grid.get_any(&AnyGridPoint::new(3, 1)), None);
    /// ```
    pub fn get_any(&self, point: &AnyGridPoint<U>) -> Option<&T> {
        self.bounded_point(point).map(|p| self.get(&p))
    }

    /// Sets the element at a location.
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
    /// grid.set(&point, 21);
    /// assert_eq!(*grid.get(&point), 21);
    /// ```
    pub fn set(&mut self, point: &GridPoint<U>, value: T) {
        *self.get_mut(point) = value;
    }

    /// Sets the element at any location, if the location is within
    /// the bounds of the grid.
    ///
    /// Returns whether or not the `point` is within the grid bound.
    /// If this was the case, then the element will have been set.
    ///
    /// # Panics
    /// This will panic if the `point` is within the grid bounds but cannot be converted to a
    /// [`GridPoint`].
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # #![feature(assert_matches)]
    /// # use std::assert_matches::assert_matches;
    /// # use aoc::prelude::*;
    /// let mut grid = Grid::<u8>::from_data(vec![vec![1, 2], vec![3, 4], vec![5, 6]]).unwrap();
    /// let point = AnyGridPoint::new(1, 1);
    ///
    /// assert_matches!(grid.get_any(&point), Some(&4));
    /// assert!(grid.set_any(&point, 21));
    /// assert_matches!(grid.get_any(&point), Some(&21));
    /// assert!(!grid.set_any(&AnyGridPoint::new(-1, 1), 21));
    /// ```
    pub fn set_any(&mut self, point: &AnyGridPoint<U>, value: T) -> bool {
        match self.bounded_point(point) {
            Some(p) => {
                self.set(&p, value);
                true
            }
            None => false,
        }
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
    /// assert_eq!(*grid.get_mut(&point), 4);
    /// *grid.get_mut(&point) = 21;
    /// assert_eq!(*grid.get_mut(&point), 21);
    /// ```
    pub fn get_mut(&mut self, point: &GridPoint<U>) -> &mut T {
        self.0
            .get_mut(point.y, point.x)
            .expect("Grid point out of bounds")
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
    /// assert_eq!(
    ///     grid.bounded_point(&AnyGridPoint::new(0, 1)),
    ///     Some(GridPoint::new(0, 1))
    /// );
    /// assert_eq!(
    ///     grid.bounded_point(&AnyGridPoint::new(1, 2)),
    ///     Some(GridPoint::new(1, 2))
    /// );
    /// assert_eq!(grid.bounded_point(&AnyGridPoint::new(0, -1)), None);
    /// assert_eq!(grid.bounded_point(&AnyGridPoint::new(2, 0)), None);
    /// ```
    pub fn bounded_point(&self, point: &AnyGridPoint<U>) -> Option<GridPoint<U>> {
        Box2D::from(self.size())
            .try_cast()
            .unwrap()
            .contains(*point)
            .then(|| point.to_usize())
    }

    /// Returns an [`Iterator`] over all valid grid points in row-major order.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use itertools::Itertools;
    ///
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
    /// assert_eq!(grid.all_points().collect_vec(), points);
    /// ```
    pub fn all_points(&self) -> impl Iterator<Item = GridPoint<U>> {
        self.size().all_points()
    }

    /// Returns an [`Iterator`] over all grid values in row-major order.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use itertools::Itertools;
    ///
    /// let grid = Grid::<u8>::from_data(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap();
    /// assert_eq!(
    ///     grid.all_values().copied().collect_vec(),
    ///     vec![1, 2, 3, 4, 5, 6]
    /// );
    /// ```
    pub fn all_values(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }

    /// Returns the underlying [`grid::Grid`] object, which features additional
    /// useful methods.
    pub fn underlying_grid(&self) -> &grid::Grid<T> {
        &self.0
    }

    /// Consumes the object and returns the [`grid::Grid`].
    pub fn into_underlying_grid(self) -> grid::Grid<T> {
        self.0
    }

    /// Returns an [`Iterator`] over the neighboring points around a `point`
    /// in row-major order such that all the points are bounded in the grid.
    ///
    /// The length of the [`Iterator`] will depend on the location of the `point`.
    /// For example, points on the edge of the grid will have fewer neighboring
    /// points than the points in the middle of the grid. The set of points may
    /// optionally include the (up to) four diagonal neighbor points as well as
    /// this `point` itself.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use itertools::Itertools;
    ///
    /// let grid = Grid::<u8>::from_data(vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]).unwrap();
    /// assert_eq!(
    ///     grid.neighbor_points(&GridPoint::new(0, 0), true, true)
    ///         .collect_vec(),
    ///     vec![
    ///         GridPoint::new(0, 0),
    ///         GridPoint::new(1, 0),
    ///         GridPoint::new(0, 1),
    ///         GridPoint::new(1, 1),
    ///     ],
    /// );
    /// assert_eq!(
    ///     grid.neighbor_points(&GridPoint::new(1, 1), false, true)
    ///         .collect_vec(),
    ///     vec![
    ///         GridPoint::new(1, 0),
    ///         GridPoint::new(0, 1),
    ///         GridPoint::new(1, 1),
    ///         GridPoint::new(2, 1),
    ///         GridPoint::new(1, 2),
    ///     ],
    /// );
    /// assert_eq!(
    ///     grid.neighbor_points(&GridPoint::new(1, 2), true, false)
    ///         .collect_vec(),
    ///     vec![
    ///         GridPoint::new(0, 1),
    ///         GridPoint::new(1, 1),
    ///         GridPoint::new(2, 1),
    ///         GridPoint::new(0, 2),
    ///         GridPoint::new(2, 2),
    ///     ],
    /// );
    /// ```
    pub fn neighbor_points<'a>(
        &'a self,
        point: &GridPoint<U>,
        include_diagonals: bool,
        include_self: bool,
    ) -> impl Iterator<Item = GridPoint<U>> + 'a {
        point
            .try_cast()
            .unwrap()
            .all_neighbor_points(include_diagonals, include_self)
            .filter_map(|p| self.bounded_point(&p))
    }

    /// Parses grid data from an array of characters.
    fn parse_data(s: &str) -> AocResult<Vec<Vec<T>>>
    where
        T: TryFrom<char>,
    {
        s.lines()
            .map(|line| {
                line.chars()
                    .map(|c| {
                        T::try_from(c).map_err(|_| {
                            AocError::Other(format!("Invalid character found: '{c}'").into())
                        })
                    })
                    .collect()
            })
            .collect()
    }

    /// Verifies that the data is not empty and returns the smallest grid
    /// size into which the data will fit.
    fn data_size(data: &[Vec<T>]) -> AocResult<GridSize<U>> {
        let width = data
            .iter()
            .map(|r| r.len())
            .max()
            .and_then(|m| (m != 0).then_some(m))
            .ok_or(AocError::Other("The grid has no content!".into()))?;

        Ok(GridSize::new(width, data.len()))
    }
}

// Additional methods for elements that have default values.
impl<T: Default + Clone, U> Grid<T, U> {
    /// Creates a default grid of a particular `size` with default values.
    ///
    /// # Panics
    /// This will panic if the `size` is invalid, that is it contains zero in either dimension.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let grid = Grid::<u8>::default(GridSize::new(3, 3));
    ///
    /// assert_eq!(
    ///     grid,
    ///     Grid::from_data(vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0]]).unwrap()
    /// );
    /// ```
    pub fn default(size: GridSize<U>) -> Self {
        size.validate();
        Self(grid::Grid::new(size.height, size.width), Default::default())
    }

    /// Creates a grid from raw data, filling in missing elements with the default
    /// value.
    ///
    /// The raw data should be a [`Vec`] of rows, with each row being
    /// itself a [`Vec`].
    /// If a `size` is not passed, then the grid size will be determined from
    /// the `data`, with the height being the number of rows and the width being
    /// the length of the longest row.
    /// If a `size` is passed, then the grid will have that size, with extraneous
    /// data discarded.
    /// In either case, any missing items in the data will be filled with the default
    /// value, noting that passed values in any shorter rows will all be on the left
    /// side of the grid followed by any default values on the right.
    /// Returns an [`AocError::Other`]
    /// if the passed data is empty.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// // This is a grid with the following values:
    /// // 0 1 0
    /// // 2 3 4
    /// // 4 0 0
    /// let grid =
    ///     Grid::<u8>::from_data_default(vec![vec![0, 1], vec![2, 3, 4], vec![4]], None).unwrap();
    /// assert_eq!(grid.size(), GridSize::new(3, 3));
    ///
    /// // This is a grid with the following values:
    /// // 0 1 0 0
    /// // 2 3 4 0
    /// // 4 0 0 0
    /// // 0 0 0 0
    /// // 0 0 0 0
    /// let grid = Grid::<u8>::from_data_default(
    ///     vec![vec![0, 1], vec![2, 3, 4], vec![4]],
    ///     Some(GridSize::new(4, 5)),
    /// )
    /// .unwrap();
    /// assert_eq!(grid.size(), GridSize::new(4, 5));
    ///
    /// // This is a grid with the following values:
    /// // 0 1 7
    /// // 2 3 4
    /// let grid = Grid::<u8>::from_data_default(
    ///     vec![vec![0, 1, 7], vec![2, 3, 4, 8], vec![4, 3]],
    ///     Some(GridSize::new(3, 2)),
    /// )
    /// .unwrap();
    /// assert_eq!(grid.size(), GridSize::new(3, 2));
    ///
    /// /// // This is a grid with the following values:
    /// // 1 0
    /// // 0 0
    /// // 4,3
    /// let grid = Grid::<u8>::from_data_default(
    ///     vec![vec![1], vec![], vec![4, 3, 8], vec![5]],
    ///     Some(GridSize::new(2, 3)),
    /// )
    /// .unwrap();
    /// assert_eq!(grid.size(), GridSize::new(2, 3));
    /// ```
    ///
    /// Invalid usage:
    /// ```
    /// # #![feature(assert_matches)]
    /// # use std::assert_matches::assert_matches;
    /// # use aoc::prelude::*;
    /// assert_matches!(
    ///     Grid::<u8>::from_data_default(vec![], None),
    ///     Err(AocError::Other(_))
    /// );
    /// ```
    pub fn from_data_default(mut data: Vec<Vec<T>>, size: Option<GridSize<U>>) -> AocResult<Self> {
        // Determine the grid size
        let size = match size {
            Some(s) => s,
            None => Self::data_size(&data)?,
        };

        // Add or remove elements to each row if necessary
        for row in data.iter_mut() {
            if size.width >= row.len() {
                for _ in 0..(size.width - row.len()) {
                    row.push(T::default());
                }
            } else {
                row.truncate(size.width)
            }
        }

        // Add or remove rows if necessary
        if size.height >= data.len() {
            for _ in 0..(size.height - data.len()) {
                data.push(vec![T::default(); size.width]);
            }
        } else {
            data.truncate(size.height)
        }

        Self::from_data(data)
    }

    /// Creates a sub-grid by cloning the applicable elements of this grid.
    ///
    /// The sub-grid location is given by the `sub_grid_box`.
    ///
    /// # Panics
    /// This will panic if any part of the sub-grid is out of the bounds of this
    /// grid, or if `sub_grid_box` has an invalid size, see [`GridSizeExt::is_valid`].
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let grid = Grid::<u8>::from_data(vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]).unwrap();
    ///
    /// assert_eq!(
    ///     grid.sub_grid(&GridBox::<GridSpace>::from_origin_and_size(
    ///         GridPoint::new(0, 0),
    ///         GridSize::new(2, 3)
    ///     )),
    ///     Grid::from_data(vec![vec![1, 2], vec![4, 5], vec![7, 8]]).unwrap(),
    /// );
    /// assert_eq!(
    ///     grid.sub_grid(&GridBox::<GridSpace>::from_origin_and_size(
    ///         GridPoint::new(1, 0),
    ///         GridSize::new(2, 2)
    ///     )),
    ///     Grid::from_data(vec![vec![2, 3], vec![5, 6]]).unwrap(),
    /// );
    /// assert_eq!(
    ///     grid.sub_grid(&GridBox::<GridSpace>::from_origin_and_size(
    ///         GridPoint::new(1, 2),
    ///         GridSize::new(2, 1)
    ///     )),
    ///     Grid::from_data(vec![vec![8, 9]]).unwrap(),
    /// );
    /// ```
    pub fn sub_grid(&self, sub_grid_box: &GridBox<U>) -> Self {
        let size = sub_grid_box.size();
        // Note that this will validate the size
        let mut out = Self::default(size);
        let shift = sub_grid_box.min.to_vector();
        for out_point in size.all_points() {
            out.set(&out_point, self.get(&(out_point + shift)).clone());
        }
        out
    }

    /// Parses the grid from a string, filling in missing values with defaults.
    ///
    /// Identical to [`Grid::from_str`] except that rows of variable lengths are
    /// allowed, with missing values being filled in with default values as is done
    /// when using [`Grid::from_data_default`].
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// # use std::str::FromStr;
    /// #[derive(Debug, Default, Clone, PartialEq, Eq)]
    /// enum Direction {
    ///     Up,
    ///     Down,
    ///     Left,
    ///     Right,
    ///     #[default]
    ///     Default,
    /// }
    /// use Direction::*;
    /// impl TryFrom<char> for Direction {
    ///     type Error = ();
    ///
    ///     fn try_from(value: char) -> Result<Self, Self::Error> {
    ///         match value {
    ///             'U' => Ok(Up),
    ///             'D' => Ok(Down),
    ///             'L' => Ok(Left),
    ///             'R' => Ok(Right),
    ///             _ => Err(()),
    ///         }
    ///     }
    /// }
    ///
    /// let string = "UDL
    ///
    /// DLRUD
    /// LR";
    /// let grid = Grid::<_, GridSpace>::from_data(vec![
    ///     vec![Up, Down, Left, Default, Default],
    ///     vec![Default, Default, Default, Default, Default],
    ///     vec![Down, Left, Right, Up, Down],
    ///     vec![Left, Right, Default, Default, Default],
    /// ])
    /// .unwrap();
    ///
    /// assert_eq!(Grid::from_str_default(string).unwrap(), grid);
    /// ```
    ///
    /// Invalid usage:
    /// ```
    /// # #![feature(assert_matches)]
    /// # use std::assert_matches::assert_matches;
    /// # use aoc::prelude::*;
    /// # use std::str::FromStr;
    /// #[derive(Debug, PartialEq, Eq)]
    /// enum Correctness {
    ///     Wrong,
    ///     Right,
    /// }
    /// use Correctness::*;
    /// impl TryFrom<char> for Correctness {
    ///     type Error = ();
    ///
    ///     fn try_from(value: char) -> Result<Self, Self::Error> {
    ///         match value {
    ///             'W' => Ok(Wrong),
    ///             'R' => Ok(Right),
    ///             _ => Err(()),
    ///         }
    ///     }
    /// }
    ///
    /// let string = "WR
    /// RWWX";
    ///
    /// assert_matches!(
    ///     Grid::<Correctness>::from_str(string),
    ///     Err(AocError::Other(_))
    /// );
    /// ```
    pub fn from_str_default(s: &str) -> AocResult<Self>
    where
        T: TryFrom<char>,
    {
        Self::parse_data(s).and_then(|data| Self::from_data_default(data, None))
    }
}

// Additional methods for clone-able elements.
impl<T: Clone> Grid<T> {
    /// Creates a [`Graph`] representation of the grid.
    ///
    /// This can be useful to, for example, find the shortest path from one point
    /// to another using [`petgraph::algo::dijkstra`](petgraph::algo::dijkstra::dijkstra).
    /// A node is created for each point in the grid, with the node weight being a
    /// clone of the corresponding element in this grid.
    ///
    /// The `edge_creator` closure can be used to create edges between adjacent points.
    /// For each point in the grid, the closure will be called for each neighbor of the point,
    /// optionally including diagonal neighbors when `include_diagonals` is `true`.
    /// The first argument of the closure will be the element of the main point, while the
    /// second argument is the element of the neighboring point.
    /// The closure should return an edge weight if an edge should be created.
    /// If [`None`] is returned, then an edge will not be created.
    /// Note that the closure will be called twice for each pair of adjacent points, once
    /// where one point is the main point, and again with other point as the main point.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use itertools::{iproduct, Itertools};
    /// use petgraph::{graph::DefaultIx, Directed};
    ///
    /// // Create a grid of numbers.
    /// let grid = Grid::from_data(vec![vec![2, 2, 3], vec![3, 1, 2], vec![4, 5, 1]]).unwrap();
    ///
    /// // Generate a corresponding graph in which directed pathways exist between adjacent
    /// // spaces only when going from one number to the same or successor number,
    /// // including diagonal neighbors. The edge pathways have no weights.
    /// //
    /// // This graph would then look as follows:
    /// // 2↔︎2→3
    /// // ↓⤪↑⤡↑
    /// // 3 1→2
    /// // ↓  ⤡↑
    /// // 4→5 1
    /// let (graph, node_grid) = grid
    ///     .as_graph::<_, Directed, DefaultIx>(true, |p, n| (n == p || *n == *p + 1).then_some(()));
    ///
    /// // Get all the node indices in row-major order.
    /// let nodes = node_grid.all_values().copied().collect_vec();
    ///
    /// // Make a list of expected edges.
    /// let expected_edges = [
    ///     // First row
    ///     (nodes[0], nodes[1]),
    ///     (nodes[0], nodes[3]),
    ///     (nodes[1], nodes[0]),
    ///     (nodes[1], nodes[2]),
    ///     (nodes[1], nodes[3]),
    ///     (nodes[1], nodes[5]),
    ///     // Second row
    ///     (nodes[3], nodes[6]),
    ///     (nodes[4], nodes[0]),
    ///     (nodes[4], nodes[1]),
    ///     (nodes[4], nodes[5]),
    ///     (nodes[4], nodes[8]),
    ///     (nodes[5], nodes[1]),
    ///     (nodes[5], nodes[2]),
    ///     // Third row
    ///     (nodes[6], nodes[7]),
    ///     (nodes[8], nodes[4]),
    ///     (nodes[8], nodes[5]),
    /// ];
    ///
    /// // Check the node weights.
    /// assert!(nodes
    ///     .iter()
    ///     .map(|n| graph.node_weight(*n).unwrap())
    ///     .copied()
    ///     .eq([2, 2, 3, 3, 1, 2, 4, 5, 1]));
    ///
    /// // Check every possible pair of nodes for the expected edges.
    /// for (a, b) in iproduct!(nodes.iter(), nodes.iter()).map(|(a, b)| (*a, *b)) {
    ///     assert!(graph.contains_edge(a, b) == expected_edges.contains(&(a, b)));
    /// }
    /// ```
    pub fn as_graph<E, Ty: EdgeType, Ix: IndexType>(
        &self,
        include_diagonals: bool,
        edge_creator: impl Fn(&T, &T) -> Option<E>,
    ) -> (Graph<T, E, Ty, Ix>, Grid<NodeIndex<Ix>>) {
        let mut graph = Graph::default();

        // Create nodes
        let node_grid = Grid::from_data(
            self.0
                .iter_rows()
                .map(|row| row.map(|t| graph.add_node(t.clone())).collect())
                .collect(),
        )
        .unwrap();

        // Create edges
        for point in node_grid.all_points() {
            for neighbor_point in node_grid.neighbor_points(&point, include_diagonals, false) {
                // Possibly add an edge
                if let Some(e) = edge_creator(self.get(&point), self.get(&neighbor_point)) {
                    let _ =
                        graph.add_edge(*node_grid.get(&point), *node_grid.get(&neighbor_point), e);
                }
            }
        }

        (graph, node_grid)
    }
}
/// Creates the grid from the underlying grid object, validating its size.
///
/// # Panics
/// This will panic if the size of the `value` is invalid, that is it contains zero in either dimension.
impl<T, U> From<grid::Grid<T>> for Grid<T, U> {
    fn from(value: grid::Grid<T>) -> Self {
        GridSize::<GridSpace>::from(value.size()).validate();

        Self(value, Default::default())
    }
}

// Additional methods for grids with boolean-like elements.
impl<T: From<bool> + Default + Clone, U> Grid<T, U> {
    /// Builds a [`Grid`] from a set of [`AnyGridPoint`]s for any value type that can be
    /// created from [`bool`] values.
    ///
    /// The size of the grid will automatically be determined in order to tightly
    /// contain all of the elements that correspond to `true`. The order of the points
    /// in the `points` [`Iterator`] makes no difference.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let grid = Grid::<u8>::from_coordinates(
    ///     [
    ///         AnyGridPoint::new(-2, -2),
    ///         AnyGridPoint::new(-1, -1),
    ///         AnyGridPoint::new(0, 0),
    ///         AnyGridPoint::new(1, 0),
    ///         AnyGridPoint::new(2, -1),
    ///         AnyGridPoint::new(3, -2),
    ///         AnyGridPoint::new(-1, 1),
    ///         AnyGridPoint::new(-2, 2),
    ///         AnyGridPoint::new(2, 1),
    ///         AnyGridPoint::new(3, 2),
    ///     ]
    ///     .iter(),
    /// );
    /// let values = vec![
    ///     vec![1, 0, 0, 0, 0, 1],
    ///     vec![0, 1, 0, 0, 1, 0],
    ///     vec![0, 0, 1, 1, 0, 0],
    ///     vec![0, 1, 0, 0, 1, 0],
    ///     vec![1, 0, 0, 0, 0, 1],
    /// ];
    ///
    /// assert_eq!(grid.size(), GridSize::new(6, 5));
    /// assert_eq!(grid, Grid::from_data(values).unwrap());
    /// ```
    pub fn from_coordinates<'a>(points: impl Iterator<Item = &'a AnyGridPoint<U>> + Clone) -> Self
    where
        U: 'a,
    {
        let x_range = points.clone().map(|p| p.x).range().unwrap_or(0..=0);
        let y_range = points.clone().map(|p| p.y).range().unwrap_or(0..=0);
        let size = GridSize::new(
            x_range.size().try_into().unwrap(),
            y_range.size().try_into().unwrap(),
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
impl<T: Into<bool> + Clone, U> Grid<T, U> {
    /// Returns a set of grid point coordinates for grid elements corresponding
    /// to `true` for value types that can be converted to [`bool`].
    ///
    /// The grid points will be those with (0, 0) as the upper left element
    /// such that every [`GridPoint`] is in the bounds of the grid.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// let (t, f) = (true, false);
    /// let values = vec![
    ///     vec![t, f, f, f, f, t],
    ///     vec![f, t, f, f, t, f],
    ///     vec![f, f, t, t, f, f],
    ///     vec![f, t, f, f, t, f],
    ///     vec![t, f, f, f, f, t],
    /// ];
    /// let grid = Grid::<bool>::from_data(values).unwrap();
    /// let coordinates = vec![
    ///     GridPoint::new(0, 0),
    ///     GridPoint::new(1, 1),
    ///     GridPoint::new(2, 2),
    ///     GridPoint::new(3, 2),
    ///     GridPoint::new(4, 1),
    ///     GridPoint::new(5, 0),
    ///     GridPoint::new(1, 3),
    ///     GridPoint::new(0, 4),
    ///     GridPoint::new(4, 3),
    ///     GridPoint::new(5, 4),
    /// ];
    ///
    /// assert_eq!(grid.as_coordinates(), coordinates.into_iter().collect());
    /// ```
    pub fn as_coordinates(&self) -> HashSet<GridPoint<U>> {
        self.all_points()
            .filter(|p| Into::<bool>::into(self.get(p).clone()))
            .collect()
    }
}

/// Parses a [`Grid`] from a string of characters with each row on a separate line.
///
/// This can be done for element types that can be fallibly converted from characters.
/// Note that the error type of the [`FromStr`] implementation for the element type
/// does not matter so it is recommended just to use the unit type. An [`Err`] will
/// be returns if not every row has the same number of characters, or if any of the
/// characters is invalid.
///
/// # Examples
/// Basic usage:
/// ```
/// # use aoc::prelude::*;
/// # use std::str::FromStr;
/// #[derive(Debug, PartialEq, Eq)]
/// enum Direction {
///     Up,
///     Down,
///     Left,
///     Right,
/// }
/// use Direction::*;
/// impl TryFrom<char> for Direction {
///     type Error = ();
///
///     fn try_from(value: char) -> Result<Self, Self::Error> {
///         match value {
///             'U' => Ok(Up),
///             'D' => Ok(Down),
///             'L' => Ok(Left),
///             'R' => Ok(Right),
///             _ => Err(()),
///         }
///     }
/// }
///
/// let string = "UDLRU
/// DLRUD
/// LRUDL";
/// let grid = Grid::from_data(vec![
///     vec![Up, Down, Left, Right, Up],
///     vec![Down, Left, Right, Up, Down],
///     vec![Left, Right, Up, Down, Left],
/// ])
/// .unwrap();
///
/// assert_eq!(Grid::from_str(string).unwrap(), grid);
/// ```
///
/// Invalid usage:
/// ```
/// # #![feature(assert_matches)]
/// # use std::assert_matches::assert_matches;
/// # use aoc::prelude::*;
/// # use std::str::FromStr;
/// #[derive(Debug, PartialEq, Eq)]
/// enum Correctness {
///     Wrong,
///     Right,
/// }
/// use Correctness::*;
/// impl TryFrom<char> for Correctness {
///     type Error = ();
///
///     fn try_from(value: char) -> Result<Self, Self::Error> {
///         match value {
///             'W' => Ok(Wrong),
///             'R' => Ok(Right),
///             _ => Err(()),
///         }
///     }
/// }
///
/// let string = "WRRW
/// RWWX";
///
/// assert_matches!(
///     Grid::<Correctness>::from_str(string),
///     Err(AocError::Other(_))
/// );
/// ```
impl<T: TryFrom<char>> FromStr for Grid<T> {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_data(s).and_then(Self::from_data)
    }
}
/// Debug display for a [`Grid`] whose elements implement [`Debug`].
impl<T: fmt::Debug, U> fmt::Debug for Grid<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = self.size();
        for y in 0..size.height {
            for x in 0..size.width {
                write!(f, "{:?}", self.get(&GridPoint::new(x, y)))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Create an object from default [`Grid`] of a particular size.
///
/// Automatically implemented for types that implement `From<Grid<T>>` for some
/// appropriate `T` that itself implements [`Default`].
pub trait GridDefault<T: Default + Clone>: From<Grid<T>> {
    /// Returns a default object from a default [`Grid`] of some `size`.
    ///
    /// # Panics
    /// This will panic if the `size` is invalid, that is either dimension is zero.
    fn default(size: GridSize) -> Self {
        Grid::default(size).into()
    }
}
impl<T: Default + Clone, O: From<Grid<T>>> GridDefault<T> for O {
    fn default(size: GridSize) -> Self {
        Self::from(Grid::default(size))
    }
}

/// Parse objects that can be created from a [`Grid`] from a grid of characters.
///
/// This is a blanket implementation on implementors of `From<Grid<T>>` for some `T`.
/// Note that we cannot just blanket implement [`FromStr`] due to the orphan rule.
pub trait FromGridStr<T>: Sized {
    /// The error type if the conversion fails.
    type Err;

    /// Creates an object from a grid of characters.
    ///
    /// Refer to [`Grid::from_str`] for examples of how grids can be parsed from
    /// strings. This returns an [`Err`] under the same conditions as
    /// [`Grid::from_str`].
    fn from_grid_str(s: &str) -> Result<Self, Self::Err>;
}
impl<T: TryFrom<char>, O: From<Grid<T>>> FromGridStr<T> for O {
    type Err = AocError;

    fn from_grid_str(s: &str) -> Result<Self, Self::Err> {
        Ok(<Grid<T> as FromStr>::from_str(s)?.into())
    }
}

/// Standard boolean [`Grid`] element that can be converted from characters,
/// where '.' is false and '#' is true.
///
/// This conversion allows grids of these elements to be parsed from strings using
/// [`Grid::from_str`].
///
/// # Examples
/// Basic usage:
/// ```
/// # use aoc::prelude::*;
/// # use aoc::grid::StdBool;
/// # use std::str::FromStr;
/// let (t, f) = (true.into(), false.into());
/// let string = ".#..
/// #.##
/// .#.#";
/// let grid =
///     Grid::<StdBool>::from_data(vec![vec![f, t, f, f], vec![t, f, t, t], vec![f, t, f, t]])
///         .unwrap();
///
/// assert_eq!(Grid::from_str(string).unwrap(), grid);
/// ```
#[derive(Deref, From, Into, Not, Default, Clone, Copy, PartialEq, Eq)]
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

/// Standard number digit [`Grid`] element that can be converted from characters,
/// where the digits can be from `0` to `9`.
///
/// This conversion allows grids of these elements to be parsed from strings using
/// [`Grid::from_str`].
///
/// # Examples
/// Basic usage:
/// ```
/// # use aoc::prelude::*;
/// # use aoc::grid::Digit;
/// # use std::str::FromStr;
/// let string = "01
/// 23
/// 45
/// 67
/// 89";
/// let grid = Grid::<Digit>::from_data(vec![
///     vec![0.into(), 1.into()],
///     vec![2.into(), 3.into()],
///     vec![4.into(), 5.into()],
///     vec![6.into(), 7.into()],
///     vec![8.into(), 9.into()],
/// ])
/// .unwrap();
///
/// assert_eq!(Grid::from_str(string).unwrap(), grid);
/// ```
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
