use std::{cmp::Eq, collections::HashSet, fmt::Debug, hash::Hash};

use super::prelude::*;
use itertools::{iproduct, Itertools};
use num::Integer;

/// Specifies elements of a CharGrid
pub type GridPoint = (usize, usize);

/// A data structed that can be represented as a 2D grid of elements
pub trait Grid: Sized {
    /// Type of each grid element.
    type Element;

    /// Size of the grid
    fn size(&self) -> (usize, usize);

    /// Get mutable reference to an element
    fn element_at(&mut self, point: &GridPoint) -> &mut Self::Element;

    /// Get element at a point
    fn get_element(&self, point: &GridPoint) -> Self::Element {
        *self.element_at(point)
    }

    /// Set element at a point
    fn set_element(&mut self, point: &GridPoint, value: Self::Element) {
        *self.element_at(point) = value;
    }

    /// Validate and convert signed point to unsigned
    fn valid_point(&self, x: isize, y: isize) -> Option<GridPoint> {
        if x >= 0 && y >= 0 {
            let x = x.try_into().unwrap();
            let y = y.try_into().unwrap();
            let size = self.size();
            if x < size.0 && y < size.1 {
                Some((x, y))
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
        Box::new(iproduct!(0..size.1, 0..size.0).map(|(y, x)| (x, y)))
    }

    /// Iterator over neighbors point
    fn neighbor_points(
        &self,
        point: &GridPoint,
        include_diagonals: bool,
        include_self: bool,
    ) -> Box<dyn Iterator<Item = GridPoint> + '_> {
        Box::new(iproduct!(-1isize..=1, -1isize..=1).filter_map(|(dy, dx)| {
            let npoint = self.valid_point(
                isize::try_from(point.0).unwrap() + dx,
                isize::try_from(point.1).unwrap() + dy,
            );
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
        }))
    }
}

/// A data structure that can be represented by a 2D grid of characters
pub trait CharGrid: Grid {
    /// Default grid of a certain size
    fn default(size: (usize, usize)) -> Self;

    /// Maps the read character to the Element.
    fn from_char(c: char) -> Option<<Self as Grid>::Element>;

    /// Maps the Element to a character for display purposes.
    fn to_char(e: &<Self as Grid>::Element) -> char;

    /// Formats the structure as a grid of characters.
    fn out_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size = self.size();
        writeln!(
            f,
            "{}",
            (0..size.1)
                .map(|y| (0..size.0)
                    .map(|x| Self::to_char(&self.get_element(&(x, y))))
                    .collect::<String>())
                .join("\n")
        )
    }

    /// Construct from a character grid.
    fn from_str(s: &str) -> AocResult<Self>
    where
        Self: Sized,
    {
        let data = s
            .lines()
            .map(|line| {
                line.chars()
                    .map(Self::from_char)
                    .collect::<Vec<Self::Element>>()
                    .into_boxed_slice()
            })
            .collect::<Vec<Box<[Self::Element]>>>()
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

        Self::from_data((width, height), data)
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

impl<C: CharGrid<Element = bool>> CharGridCoordinates for C {
    fn to_coordinates<N>(&self) -> HashSet<(N, N)>
    where
        N: TryFrom<usize> + Hash + Eq,
        <N as TryFrom<usize>>::Error: Debug,
    {
        let data = self.to_data();
        iproduct!(0..data[0].len(), 0..data.len())
            .filter(|(x, y)| data[*y][*x])
            .map(|(x, y)| -> (N, N) { (N::try_from(x).unwrap(), N::try_from(y).unwrap()) })
            .collect()
    }

    fn from_coordinates<N>(points: &HashSet<(N, N)>) -> AocResult<Self>
    where
        N: Integer + Copy + Clone + TryInto<usize> + TryFrom<usize> + Eq + Hash,
        <N as TryInto<usize>>::Error: Debug,
        <N as TryFrom<usize>>::Error: Debug,
        Self: Sized,
    {
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
        Self::from_data((width, height), data)
    }
}
