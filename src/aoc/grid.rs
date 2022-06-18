use core::slice::SlicePattern;
use std::{cmp::Eq, collections::HashSet, hash::Hash, num::TryFromIntError};

use super::prelude::*;
use cgmath::Vector2;
use itertools::{iproduct, Itertools};

/// Specifies elements of a Grid
pub type GridPoint = Vector2<usize>;
/// Specifies sizes of a Grid
pub type GridSize = Vector2<usize>;

/// Useful trait to convert between point types since we cannot implement the std trait
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

/// A grid of arbitrary data
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
    /// Size of the grid
    pub fn size(&self) -> &GridSize {
        &self.size
    }

    /// Get element at a point
    pub fn get(&self, point: &GridPoint) -> &T {
        &self.data[point.y][point.x]
    }

    /// Set element at a point
    pub fn set(&mut self, point: &GridPoint, value: T) {
        *self.element_at(point) = value;
    }

    /// Get mut reference to an element
    pub fn element_at(&mut self, point: &GridPoint) -> &mut T {
        &mut self.data[point.y][point.x]
    }

    /// From data with verification
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

    /// Validate and convert signed point to unsigned
    pub fn valid_point(&self, point: &Vector2<isize>) -> Option<GridPoint> {
        if point.x >= 0 && point.y >= 0 {
            let x = point.x.try_into().unwrap();
            let y = point.y.try_into().unwrap();
            let size = self.size();
            if x < size.x && y < size.y {
                Some(GridPoint::new(x, y))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Iterate over all points
    pub fn all_points(&self) -> impl Iterator<Item = GridPoint> {
        let size = self.size();
        Box::new(iproduct!(0..size.y, 0..size.x).map(|(y, x)| GridPoint::new(x, y)))
    }

    /// Iterate over all values
    pub fn all_values(&self) -> impl Iterator<Item = &T> {
        Box::new(self.all_points().map(|p| self.get(&p)))
    }

    /// Iterate over a row
    pub fn row_iter(&self, row: usize) -> impl Iterator<Item = &T> {
        self.data[row].iter()
    }

    /// Iterator over column
    pub fn col_iter(&self, col: usize) -> impl Iterator<Item = &T> {
        (0..self.size.y).map(move |y| &self.data[y][col])
    }

    /// Iterator over all rows as slices
    pub fn rows_iter(&self) -> impl Iterator<Item = &[T]> {
        self.data.iter().map(|row| row.as_slice())
    }

    /// Iterator over neighbors point
    pub fn neighbor_points<'a>(
        &'a self,
        point: &'a GridPoint,
        include_diagonals: bool,
        include_self: bool,
    ) -> impl Iterator<Item = GridPoint> + 'a {
        iproduct!(-1isize..=1, -1isize..=1).filter_map(move |(dy, dx)| {
            let npoint = self.valid_point(&Vector2::new(
                isize::try_from(point.x).unwrap() + dx,
                isize::try_from(point.y).unwrap() + dy,
            ));
            if dx == 0 && dy == 0 {
                if include_self {
                    npoint
                } else {
                    None
                }
            } else if !include_diagonals && (dx + dy).abs() != 1 {
                None
            } else {
                npoint
            }
        })
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

/// A data structure that can be represented by a 2D grid of characters
pub trait CharGrid<T> {
    /// Maps the read character to the Element.
    fn from_char(c: char) -> Option<T>;

    /// Maps the Element to a character for display purposes.
    fn to_char(e: &T) -> char;

    /// Retrieve the Grid
    fn get_grid(&self) -> &Grid<T>;

    /// Formats the structure as a grid of characters.
    fn out_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let grid = self.get_grid();
        let size = grid.size();
        writeln!(
            f,
            "{}",
            (0..size.y)
                .map(|y| (0..size.x)
                    .map(|x| Self::to_char(grid.get(&GridPoint::new(x, y))))
                    .collect::<String>())
                .join("\n")
        )
    }

    /// Construct from a character grid.
    fn grid_from_str(s: &str) -> AocResult<Grid<T>> {
        let data = s
            .lines()
            .map(|line| {
                line.chars()
                    .map(|c| {
                        Self::from_char(c).ok_or_else(|| {
                            AocError::InvalidInput(
                                format!("Invalid character found: '{}'", c).into(),
                            )
                        })
                    })
                    .collect()
            })
            .collect::<Result<_, _>>()?;
        Grid::from_data(data)
    }
}

impl Grid<bool> {
    pub fn to_coordinates(&self) -> HashSet<GridPoint> {
        self.all_points().filter(|p| *self.get(p)).collect()
    }

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
            grid.set(&point, true);
        }
        grid
    }
}
impl CharGrid<bool> for Grid<bool> {
    fn get_grid(&self) -> &Grid<bool> {
        self
    }

    fn from_char(c: char) -> Option<bool> {
        match c {
            '#' => Some(true),
            '.' => Some(false),
            _ => None,
        }
    }

    fn to_char(e: &bool) -> char {
        if *e {
            '#'
        } else {
            '.'
        }
    }
}
impl core::fmt::Debug for Grid<bool> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.out_fmt(f)
    }
}

/// Grid for single digit numbers
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
}
