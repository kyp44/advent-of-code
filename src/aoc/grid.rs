//! 2D grids of values.
//!
//! Contains the main [`Grid`] struct, associated traits, and some useful
//! grid element types.

use super::prelude::*;
use core::slice::SlicePattern;
use derive_more::{Add, AddAssign, Deref, From, Into, Not, Sub, SubAssign};
use euclid::{Box2D, Point2D, Size2D, Vector2D};
use itertools::{iproduct, process_results};
use num::FromPrimitive;
use petgraph::{graph::NodeIndex, stable_graph::IndexType, EdgeType, Graph};
use std::{cmp::Eq, collections::HashSet, fmt, hash::Hash, str::FromStr};

/// A grid coordinate system in which the origin is the in upper left of the grid
/// and increasing `y` moves down in the grid.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridSpace;

/// The size of a [`Grid`].
///
/// Sizes in which either element is zero are not valid.
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
            panic!("Grid size is invalid");
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
    /// [fundamental limitation of RPITIT](https://users.rust-lang.org/t/fully-owned-iterator-causing-lifetime-problems/107677/4).
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
    /// let size = GridSize::new(5, 4);
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
    /// assert_eq!(
    ///     AnyGridPoint::new(0, 0).all_neighbor_points(true, true).collect::<Vec<_>>(),
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
    ///     AnyGridPoint::new(-4, -2).all_neighbor_points(true, false).collect::<Vec<_>>(),
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
    ///     AnyGridPoint::new(5, 6).all_neighbor_points(false, false).collect::<Vec<_>>(),
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
        let any_size = size.try_cast().unwrap();

        AnyGridPoint::new(
            self.x.rem_euclid(any_size.width),
            self.y.rem_euclid(any_size.height),
        )
        .to_usize()
    }

    fn all_neighbor_points(
        &self,
        include_diagonals: bool,
        include_self: bool,
    ) -> Self::NeighborPoints {
        // TODO can we just deference?
        let point = self.clone();

        iproduct!(-1isize..=1, -1isize..=1).filter_map(move |(dy, dx)| {
            let point = point + Vector2D::new(dx, dy);
            // TODO use manhattan distance here!
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
}

/// A 2D grid of values.
///
/// The values are addressed by a [`GridPoint`].
/// Conceptually, the origin point (0, 0) is in the upper-left corner of the grid
/// with increasing `x` moving to the right, and increasing `y` moving down.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Grid<T, U = GridSpace> {
    /// The size of the grid.
    size: GridSize<U>,
    /// The actual grid data of fixed size.
    ///
    /// This is an array of arrays in which each inner array is a row and is
    /// guaranteed to have the same length, which is of course the width of
    /// the grid.
    data: Box<[Box<[T]>]>,
}
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
    /// assert_eq!(grid, Grid::from_data(vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0]]).unwrap());
    /// ```
    pub fn default(size: GridSize<U>) -> Self {
        size.validate();
        Self {
            size,
            data: vec![vec![T::default(); size.width].into_boxed_slice(); size.height]
                .into_boxed_slice(),
        }
    }
}
impl<T, U> Grid<T, U> {
    /// Creates a grid from raw data.
    ///
    /// The raw data should be a [`Vec`] for rows, with each row being
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
    /// let result = Grid::from_data(vec![
    ///     vec![1, 2, 3],
    ///     vec![4, 5],
    ///     vec![6],
    /// ]);
    /// assert_matches!(result, Err(AocError::Other(_)));
    /// ```
    pub fn from_data(data: Vec<Vec<T>>) -> AocResult<Self> {
        // Verify that we have at least one row
        let height = data.len();
        if height < 1 {
            return Err(AocError::Other("The grid has no content!".into()));
        }

        // Verify that all the row widths are the same
        let width = data[0].len();
        let data = process_results(
            data.into_iter().map(|row| {
                if row.len() != width {
                    Err(AocError::Other(
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
    pub fn size(&self) -> &GridSize<U> {
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
    pub fn get(&self, point: &GridPoint<U>) -> &T {
        &self.data[point.y][point.x]
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
        *self.element_at(point) = value;
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
    /// assert_eq!(*grid.element_at(&point), 4);
    /// *grid.element_at(&point) =  21;
    /// assert_eq!(*grid.element_at(&point), 21);
    /// ```
    pub fn element_at(&mut self, point: &GridPoint<U>) -> &mut T {
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
    /// assert_eq!(grid.bounded_point(&AnyGridPoint::new(0, 1)), Some(GridPoint::new(0, 1)));
    /// assert_eq!(grid.bounded_point(&AnyGridPoint::new(1, 2)), Some(GridPoint::new(1, 2)));
    /// assert_eq!(grid.bounded_point(&AnyGridPoint::new(0, -1)), None);
    /// assert_eq!(grid.bounded_point(&AnyGridPoint::new(2, 0)), None);
    /// ```
    pub fn bounded_point(&self, point: &AnyGridPoint<U>) -> Option<GridPoint<U>> {
        Box2D::from(self.size)
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
    pub fn all_points(&self) -> impl Iterator<Item = GridPoint<U>> {
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
        (0..self.size.height).map(move |y| &self.data[y][column])
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
    /// let grid = Grid::<u8>::from_data(vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]).unwrap();
    ///
    /// assert_eq!(
    ///     grid.neighbor_points(&GridPoint::new(0, 0), true, true).collect::<Vec<_>>(),
    ///     vec![
    ///         GridPoint::new(0, 0),
    ///         GridPoint::new(1, 0),
    ///         GridPoint::new(0, 1),
    ///         GridPoint::new(1, 1),
    ///     ],    
    /// );
    /// assert_eq!(
    ///     grid.neighbor_points(&GridPoint::new(1, 1), false, true).collect::<Vec<_>>(),
    ///     vec![
    ///         GridPoint::new(1, 0),
    ///         GridPoint::new(0, 1),
    ///         GridPoint::new(1, 1),
    ///         GridPoint::new(2, 1),
    ///         GridPoint::new(1, 2),
    ///     ],    
    /// );
    /// assert_eq!(
    ///     grid.neighbor_points(&GridPoint::new(1, 2), true, false).collect::<Vec<_>>(),
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

    /// Creates a sub-grid by cloning the applicable elements of this grid.
    ///
    /// The inclusive sub-grid location is given by the `sub_grid_box`.
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
    ///     grid.sub_grid(&GridPoint::new(0, 0), GridSize::new(2, 3)),
    ///     Grid::from_data(vec![vec![1, 2], vec![4, 5], vec![7, 8]]).unwrap(),
    /// );
    /// assert_eq!(
    ///     grid.sub_grid(&GridPoint::new(1, 0), GridSize::new(2, 2)),
    ///     Grid::from_data(vec![vec![2, 3], vec![5, 6]]).unwrap(),
    /// );
    /// assert_eq!(
    ///     grid.sub_grid(&GridPoint::new(1, 2), GridSize::new(2, 1)),
    ///     Grid::from_data(vec![vec![8, 9]]).unwrap(),
    /// );
    /// ```
    /// TODO Update documentation and test code.
    pub fn sub_grid(&self, sub_grid_box: GridBox<U>) -> Self
    where
        T: Default + Clone,
    {
        let size = sub_grid_box.size();
        // Note that this will validate the size
        let mut out = Self::default(size);
        let shift = sub_grid_box.min.to_vector();
        for out_point in size.all_points() {
            out.set(&out_point, self.get(&(out_point + shift)).clone());
        }
        out
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
    /// use itertools::{Itertools, iproduct};
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
            self.rows_iter()
                .map(|row| row.iter().map(|t| graph.add_node(t.clone())).collect())
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
    /// let grid = Grid::<u8>::from_coordinates([
    ///     AnyGridPoint::new(-2, -2),
    ///     AnyGridPoint::new(-1, -1),
    ///     AnyGridPoint::new(0, 0),
    ///     AnyGridPoint::new(1, 0),
    ///     AnyGridPoint::new(2, -1),
    ///     AnyGridPoint::new(3, -2),
    ///     AnyGridPoint::new(-1, 1),
    ///     AnyGridPoint::new(-2, 2),
    ///     AnyGridPoint::new(2, 1),
    ///     AnyGridPoint::new(3, 2),
    /// ].iter());
    /// let values = vec![
    ///     vec![1, 0, 0, 0, 0, 1],
    ///     vec![0, 1, 0, 0, 1, 0],
    ///     vec![0, 0, 1, 1, 0, 0],
    ///     vec![0, 1, 0, 0, 1, 0],
    ///     vec![1, 0, 0, 0, 0, 1],
    /// ];
    ///
    /// assert_eq!(grid.size(), &GridSize::new(6, 5));
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
/// ]).unwrap();
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
/// assert_matches!(Grid::<Correctness>::from_str(string), Err(AocError::Other(_)));
/// ```
impl<T: TryFrom<char>> FromStr for Grid<T> {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = s
            .lines()
            .map(|line| {
                line.chars()
                    .map(|c| {
                        T::try_from(c).map_err(|_| {
                            AocError::Other(format!("Invalid character found: '{c}'").into())
                        })
                    })
                    .collect()
            })
            .collect::<Result<_, _>>()?;
        Self::from_data(data)
    }
}
/// Debug display for a [`Grid`] whose elements implement [`Debug`].
impl<T: fmt::Debug> fmt::Debug for Grid<T> {
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
/// let grid = Grid::<StdBool>::from_data(vec![
///     vec![f, t, f, f],
///     vec![t, f, t, t],
///     vec![f, t, f, t],
/// ]).unwrap();
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
/// ]).unwrap();
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
