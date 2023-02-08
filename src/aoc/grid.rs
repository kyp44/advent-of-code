use super::prelude::*;
use cgmath::Vector2;
use core::slice::SlicePattern;
use derive_more::{Deref, From, Into};
use itertools::{iproduct, Itertools};
use std::{cmp::Eq, collections::HashSet, fmt, hash::Hash, str::FromStr};

// Specifies elements of a Grid
pub type GridPoint = Vector2<usize>;
// Specifies sizes of a Grid
pub type GridSize = Vector2<usize>;

// Useful trait to convert between point types since we cannot implement the std trait
pub trait PointTryInto<T> {
    type Error;

    fn try_point_into(self) -> Result<T, Self::Error>;
}
impl<A, B> PointTryInto<Vector2<B>> for Vector2<A>
where
    B: TryFrom<A>,
{
    type Error = B::Error;

    fn try_point_into(self) -> Result<Vector2<B>, Self::Error> {
        Ok(Vector2::new(self.x.try_into()?, self.y.try_into()?))
    }
}

// Extensions for GridSize
pub trait GridSizeExt {
    fn all_points(&self) -> Box<dyn Iterator<Item = GridPoint>>;
}
impl GridSizeExt for GridSize {
    // Iterator over all grid points in row major order.
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
    pub fn valid_point(&self, point: &Vector2<isize>) -> Option<GridPoint> {
        if point.x >= 0 && point.y >= 0 {
            let point: GridPoint = point.try_point_into().unwrap();
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
        point: Vector2<isize>,
        include_diagonals: bool,
        include_self: bool,
    ) -> impl Iterator<Item = Vector2<isize>> {
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
            point.try_point_into().unwrap(),
            include_diagonals,
            include_self,
        )
        .filter_map(|p| self.valid_point(&p))
    }

    pub fn sub_grid(&self, point: &GridPoint, size: GridSize) -> Self
    where
        T: Default + Clone,
    {
        let mut out = Self::default(size);
        for out_point in out.all_points() {
            out.set(&out_point, self.get(&(*point + out_point)).clone());
        }
        out
    }
}

/// Parsing a grid from a grid of characters for element types that can be fallibly
/// converted from characters.
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

/// Create an object from a default [`Grid`].
pub trait GridDefault<T: Default + Clone>: From<Grid<T>> {
    /// The a default object from a default [`Grid`] of some `size`.
    fn default(size: GridSize) -> Self {
        Grid::default(size).into()
    }
}

/// Parse from a string of a grid characters with each row on a separate line.
///
/// Note that we cannot just blanket implement [`FromStr`] due to the orphan rule.
trait GridFromStr<T>: Sized {
    /// The error type if the conversion fails.
    type Err;

    /// Create from a grid of characters.
    fn from_str(s: &str) -> Result<Self, Self::Err>;
}
impl<T: TryFrom<char>, O: From<Grid<T>>> GridFromStr<T> for O {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(<Grid<T> as FromStr>::from_str(s)?.into())
    }
}

// TODO: Can we add a from_grid required method and then implement FromStr?
// This would be useful at least in the 2020 day 17 problem, but would need
// to go look for other use cases.
// This might makes sense to implement From<str> when From<Grid> is implemented.
// This does not compile and we might need to post it somewhere as it's not clear
// how to accomplish this.
//impl<T, S: From<Grid<T>> + CharGrid<T>> FromStr for S {}
// I think the best way to refactor this is perhaps to generally use the Newtype
// pattern for grid elements so that a character conversion can be implemented for
// these. The current CharGrid trait is weird in that it's implemented on the grid
// as a whole. There may be crates that help with the Newtype pattern or we could do
// our own in terms of easily working with the underlying value. Also we could have
// a general Digit(u8) for most of our u8 grids, and maybe something similar for the
// current Grid<bool> implementations, though or maybe some special trait for boolean-
// like Newtypes for the useful methods there (e.g. as_coordinates). Maybe there is some
// Kind of Repr crate we could use (or we could write our own to easily access the
// underlying value). Look at the shrinkwraprs crate.

#[derive(Deref, From, Into, Default, Clone, Copy)]
struct StdBool(bool);
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

// Additional traits for grids with boolean-like elements.
impl<T: From<bool> + Default + Clone> Grid<T> {
    pub fn from_coordinates(points: impl Iterator<Item = Vector2<isize>> + Clone) -> Self {
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

/*
// Grid for single digit numbers
impl CharGrid<u8> for Grid<u8> {
    fn get_grid(&self) -> &Grid<u8> {
        self
    }

    fn from_char(c: char) -> Option<u8> {
        c.to_digit(10).map(|d| d.try_into().unwrap())
    }

    fn to_char(e: &u8) -> char {
        char::from_digit((*e).into(), 10).unwrap()
    }
}
impl core::fmt::Debug for Grid<u8> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.out_fmt(f)
    }
} */
