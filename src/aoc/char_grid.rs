use std::{cmp::Eq, collections::HashSet, fmt::Debug, hash::Hash};

use super::prelude::*;
use itertools::{iproduct, Itertools};
use num::Integer;

/// A data structure that can be represented by a grid of characters
pub trait CharGrid {
    /// Type of each grid element.
    type Element;

    /// Maps the read character to the Element.
    fn from_char(c: char) -> Self::Element;

    /// Maps the Element to a character for display purposes.
    fn to_char(e: &Self::Element) -> char;

    /// Creates the structure from a read grid.
    fn from_data(size: (usize, usize), data: Box<[Box<[Self::Element]>]>) -> AocResult<Self>
    where
        Self: Sized;

    /// Supplies the grid data.
    fn to_data(&self) -> &[Box<[Self::Element]>];

    /// Formats the structure as a grid of characters.
    fn out_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            self.to_data()
                .iter()
                .map(|row| row.iter().map(|e| Self::to_char(e)).collect::<String>())
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
        let width = (x_range.end - x_range.start).try_into().unwrap();
        let height = (y_range.end - y_range.start).try_into().unwrap();

        let mut data = vec![vec![false; width].into_boxed_slice(); height].into_boxed_slice();

        for point in iproduct!(0..width, 0..height) {
            let x = N::try_from(point.0).unwrap() + x_range.start;
            let y = N::try_from(point.1).unwrap() + y_range.start;
            if points.contains(&(x, y)) {
                data[point.1][point.0] = true;
            }
        }
        Self::from_data((width, height), data)
    }
}
