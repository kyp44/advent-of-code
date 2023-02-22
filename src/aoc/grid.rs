//! 2D grids of values.
//!
//! Contains the main [`Grid`] struct, associated traits, and some useful
//! grid element types.
use super::prelude::*;
use cgmath::{EuclideanSpace, Point2, Vector2};
use core::slice::SlicePattern;
use derive_more::{Add, AddAssign, Deref, From, Into, Not, Sub, SubAssign};
use itertools::{iproduct, Itertools};
use num::FromPrimitive;
use std::{cmp::Eq, collections::HashSet, fmt, hash::Hash, str::FromStr};

/// A point location in a [`Grid`].
pub type GridPoint = Point2<usize>;
/// The size of a [`Grid`].
pub type GridSize = Vector2<usize>;

/// Extension trait for [`GridSize`].
pub trait GridSizeExt {
    /// Returns an [`Iterator`] over all points in a grid of this size in row-major order
    ///
    /// # Example
    ///
    /// ```
    /// use aoc::prelude::*;
    ///
    /// let size = GridSize::new(2, 3);
    /// let points = size.all_points().collect::<Vec<_>>();
    ///
    /// assert_eq!(points, vec![
    ///     GridPoint::new(0, 0),
    ///     GridPoint::new(0, 1),
    /// ]);
    /// ```
    fn all_points(&self) -> Box<dyn Iterator<Item = GridPoint>>;
}
impl GridSizeExt for GridSize {
    fn all_points(&self) -> Box<dyn Iterator<Item = GridPoint>> {
        Box::new(iproduct!(0..self.y, 0..self.x).map(|(y, x)| GridPoint::new(x, y)))
    }
}

// A grid of arbitrary data
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Grid<T> {
    size: GridSize,
    data: Box<[Box<[T]>]>,
}
impl<T: Default + Clone> Grid<T> {
    pub fn default(size: GridSize) -> Self {
        Self {
            size,
            data: vec![vec![T::default(); size.x].into_boxed_slice(); size.y].into_boxed_slice(),
        }
    }
}
impl<T> Grid<T> {
    // Size of the grid
    pub fn size(&self) -> &GridSize {
        &self.size
    }

    // Get element at a point
    pub fn get(&self, point: &GridPoint) -> &T {
        &self.data[point.y][point.x]
    }

    // Set element at a point
    pub fn set(&mut self, point: &GridPoint, value: T) {
        *self.element_at(point) = value;
    }

    // Get mut reference to an element
    pub fn element_at(&mut self, point: &GridPoint) -> &mut T {
        &mut self.data[point.y][point.x]
    }

    // From data with verification
    pub fn from_data(data: Box<[Box<[T]>]>) -> AocResult<Self> {
        // Verify that we have at least one row
        let height = data.len();
        if height < 1 {
            return Err(AocError::InvalidInput("The grid has no content!".into()));
        }

        // Verify that all the row widths are the same
        let width = data[0].len();
        for row in data.iter() {
            if row.len() != width {
                return Err(AocError::InvalidInput(
                    format!(
                        "Grid row has a length of {} instead of the expected {}",
                        row.len(),
                        width
                    )
                    .into(),
                ));
            }
        }

        Ok(Self {
            size: GridSize::new(width, height),
            data,
        })
    }

    // Validate and convert signed point to unsigned
    pub fn valid_point(&self, point: &Point2<isize>) -> Option<GridPoint> {
        if point.x >= 0 && point.y >= 0 {
            let point: GridPoint = Point2::try_point_from(*point).unwrap();
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

    // Iterator over all points
    pub fn all_points(&self) -> impl Iterator<Item = GridPoint> {
        self.size().all_points()
    }

    // Iterate over all values
    pub fn all_values(&self) -> impl Iterator<Item = &T> {
        Box::new(self.all_points().map(|p| self.get(&p)))
    }

    // Iterate over a row
    pub fn row_iter(&self, row: usize) -> impl Iterator<Item = &T> {
        self.data[row].iter()
    }

    // Iterator over column
    pub fn col_iter(&self, col: usize) -> impl Iterator<Item = &T> {
        (0..self.size.y).map(move |y| &self.data[y][col])
    }

    // Iterator over all rows as slices
    pub fn rows_iter(&self) -> impl Iterator<Item = &[T]> {
        self.data.iter().map(|row| row.as_slice())
    }

    // Iterate over all neighboring points in row major order, even points not in the grid.
    pub fn all_neighbor_points(
        &self,
        point: Point2<isize>,
        include_diagonals: bool,
        include_self: bool,
    ) -> impl Iterator<Item = Point2<isize>> {
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

    // Iterate over neighboring points in row major order.
    pub fn neighbor_points<'a>(
        &'a self,
        point: &GridPoint,
        include_diagonals: bool,
        include_self: bool,
    ) -> impl Iterator<Item = GridPoint> + 'a {
        self.all_neighbor_points(
            Point2::try_point_from(*point).unwrap(),
            include_diagonals,
            include_self,
        )
        .filter_map(|p| self.valid_point(&p))
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
    pub fn from_coordinates(points: impl Iterator<Item = Point2<isize>> + Clone) -> Self {
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
