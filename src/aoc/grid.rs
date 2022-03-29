use std::{cmp::Eq, collections::HashSet, fmt::Debug, hash::Hash};

use super::prelude::*;
use cgmath::Vector2;
use itertools::{iproduct, Itertools};
use num::Integer;

/// Specifies elements of a CharGrid
pub type GridPoint = Vector2<usize>;
pub type GridSize = Vector2<usize>;

/// A data structed that can be represented as a 2D grid of elements
pub trait Grid<E>: Sized {
    /// Default grid of a certain size
    fn default(size: GridSize) -> Self;

    /// Size of the grid
    fn size(&self) -> &GridSize;

    /// Get element at a point
    fn get(&self, point: &GridPoint) -> &E;

    /// Set element at a point
    fn set(&mut self, point: &GridPoint, value: E);

    /// Create from data
    fn from_data(data: impl Iterator<Item = impl Iterator<Item = E>>) -> Self {
        // TODO
        let mut grid = Self::default(GridSize::new(1, 1));
        grid
    }

    /// Validate and convert signed point to unsigned
    fn valid_point(&self, point: &Vector2<isize>) -> Option<GridPoint> {
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
    fn all_points(&self) -> Box<dyn Iterator<Item = GridPoint>> {
        let size = self.size();
        Box::new(iproduct!(0..size.y, 0..size.x).map(|(y, x)| GridPoint::new(x, y)))
    }

    /// Iterator over neighbors point
    fn neighbor_points<'a>(
        &'a self,
        point: &'a GridPoint,
        include_diagonals: bool,
        include_self: bool,
    ) -> Box<dyn Iterator<Item = GridPoint> + 'a> {
        Box::new(
            iproduct!(-1isize..=1, -1isize..=1).filter_map(move |(dy, dx)| {
                // TODO
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
            }),
        )
    }
}

/// A data structure that can be represented by a 2D grid of characters
pub trait CharGrid<E>: Grid<E> {
    /// Maps the read character to the Element.
    fn from_char(c: char) -> Option<E>;

    /// Maps the Element to a character for display purposes.
    fn to_char(e: &E) -> char;

    /// Formats the structure as a grid of characters.
    fn out_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size = self.size();
        writeln!(
            f,
            "{}",
            (0..size.y)
                .map(|y| (0..size.x)
                    .map(|x| Self::to_char(&self.get(&GridPoint::new(x, y))))
                    .collect::<String>())
                .join("\n")
        )
    }

    /// Construct from a character grid.
    fn from_str(s: &str) -> AocResult<Self>
    where
        Self: Sized,
    {
        todo!()
        /*
        let data = s
            .lines()
            .map(|line| {
                Ok(line
                    .chars()
                    .map(|c| {
                        Self::from_char(c).ok_or_else(|| {
                            AocError::InvalidInput(
                                format!("Invalid character found: '{}'", c).into(),
                            )
                        })
                    })
                    .collect::<AocResult<Vec<E>>>()?
                    .into_boxed_slice())
            })
            .collect::<AocResult<Vec<Box<[E]>>>>()?
            .into_boxed_slice();

        // Verify that all of the rows have the same width
        let height = data.len();
        let err = Err(AocError::InvalidInput("The grid has no content!".into()));
        if height < 1 {
            return err;
        }
        let width = data[0].len();
        if width < 1 {
            return err;
        }
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

        Self::from_data((width, height), data)*/
    }
}

/// A boolean CharGrid can be alternatively represented as a set of coordinates.
pub trait CharGridCoordinates {
    /// Determine the 2D coordinates of the true cells, with the indices
    /// Being the coordinates in the boxed data.
    fn to_coordinates<N>(&self) -> HashSet<(N, N)>
    where
        N: TryFrom<usize> + Hash + Eq,
        <N as TryFrom<usize>>::Error: Debug;

    /// Construct the grid from 2D coordinates of set cells, where the size is
    /// determined from from the min and max coordinates. If the set is empty,
    /// then the grid with be 1x1 with the single cell being unset.
    fn from_coordinates<N>(points: &HashSet<(N, N)>) -> AocResult<Self>
    where
        N: Integer + Copy + Clone + TryInto<usize> + TryFrom<usize> + Eq + Hash,
        <N as TryInto<usize>>::Error: Debug,
        <N as TryFrom<usize>>::Error: Debug,
        Self: Sized;
}

impl<C: CharGrid<bool>> CharGridCoordinates for C {
    fn to_coordinates<N>(&self) -> HashSet<(N, N)>
    where
        N: TryFrom<usize> + Hash + Eq,
        <N as TryFrom<usize>>::Error: Debug,
    {
        todo!()
        /*let data = self.to_data();
        iproduct!(0..data[0].len(), 0..data.len())
            .filter(|(x, y)| data[*y][*x])
            .map(|(x, y)| -> (N, N) { (N::try_from(x).unwrap(), N::try_from(y).unwrap()) })
            .collect()*/
    }

    fn from_coordinates<N>(points: &HashSet<(N, N)>) -> AocResult<Self>
    where
        N: Integer + Copy + Clone + TryInto<usize> + TryFrom<usize> + Eq + Hash,
        <N as TryInto<usize>>::Error: Debug,
        <N as TryFrom<usize>>::Error: Debug,
        Self: Sized,
    {
        todo!()
        /*
        let x_range = points.iter().map(|p| p.0).range();
        let y_range = points.iter().map(|p| p.1).range();
        let width = match x_range {
            Some(ref r) => r.len().try_into().unwrap(),
            None => 1,
        };
        let height = match y_range {
            Some(ref r) => r.len().try_into().unwrap(),
            None => 1,
        };

        let mut data = vec![vec![false; width].into_boxed_slice(); height].into_boxed_slice();

        // Set any points
        if let (Some(xr), Some(yr)) = (x_range, y_range) {
            for point in iproduct!(0..width, 0..height) {
                let x = N::try_from(point.0).unwrap() + *xr.start();
                let y = N::try_from(point.1).unwrap() + *yr.start();
                if points.contains(&(x, y)) {
                    data[point.1][point.0] = true;
                }
            }
        }
        Self::from_data((width, height), data)*/
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct BasicGrid<T> {
    size: GridSize,
    grid: Box<[Box<[T]>]>,
}
impl<T: Default + Clone> Grid<T> for BasicGrid<T> {
    fn default(size: GridSize) -> Self {
        Self {
            size,
            grid: vec![vec![T::default(); size.x].into_boxed_slice(); size.y].into_boxed_slice(),
        }
    }

    fn size(&self) -> &GridSize {
        &self.size
    }

    fn get(&self, point: &GridPoint) -> &T {
        &self.grid[point.y][point.x]
    }

    fn set(&mut self, point: &GridPoint, value: T) {
        self.grid[point.y][point.x] = value;
    }
}
