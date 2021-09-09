use super::prelude::*;
use itertools::Itertools;

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

    /// Supplies the grid data for display purposes.
    fn to_data(&self) -> Option<&[Box<[Self::Element]>]> {
        None
    }

    /// Formats the structure as a grid of characters.
    fn out_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            self.to_data()
                .unwrap()
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
                    .map(|c| Self::from_char(c))
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

        Ok(Self::from_data((width, height), data)?)
    }
}
