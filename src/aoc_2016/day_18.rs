use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "num rows: 3
    first row: ..^^.";
            answers = unsigned![6];
        }
        example {
            input = "num rows: 10
    first row: .^^.^.^^^^";
            answers = unsigned![38];
        }
        actual_answers = unsigned![1951, 20002936];
    }
}

/// Contains solution implementation items.
mod solution {
    use aoc::parse::field_line_parser;
    use nom::{branch::alt, bytes::complete::tag, combinator::map, multi::many1};
    use std::fmt::Write;

    use super::*;

    /// A tile in the [`TrappedRoom`].
    ///
    /// Can be parsed from text input.
    #[derive(Clone, Copy)]
    pub enum Tile {
        /// A safe tile.
        Safe,
        /// A trapped tile.
        Trap,
    }
    impl Tile {
        /// Returns whether or not the tile is trapped.
        pub fn is_trap(&self) -> bool {
            match self {
                Tile::Safe => false,
                Tile::Trap => true,
            }
        }
    }
    impl Parsable for Tile {
        type Parsed<'a> = Self;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            alt((map(tag("."), |_| Self::Safe), map(tag("^"), |_| Self::Trap))).parse(input)
        }
    }
    impl TryFrom<char> for Tile {
        type Error = AocError;

        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                '.' => Ok(Self::Safe),
                '^' => Ok(Self::Trap),
                _ => Err(AocError::InvalidInput(
                    format!("'{value}' is not a valid tile character").into(),
                )),
            }
        }
    }
    impl std::fmt::Debug for Tile {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_char(match self {
                Self::Safe => '.',
                Self::Trap => '^',
            })
        }
    }

    /// A single row of [`Tile`]s.
    ///
    /// Can be parsed from text input.
    #[derive(Clone)]
    pub struct TileRow(Vec<Tile>);
    impl Parsable for TileRow {
        type Parsed<'a> = Self;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(many1(Tile::parser), Self).parse(input)
        }
    }
    impl TileRow {
        /// Returns the left, center, and right tiles, respectively, for a given
        /// index `idx`.
        ///
        /// Tiles outside the room are considered safe.
        ///
        /// # Panics
        /// This will panic if `idx` is out of range.
        fn neighboring_tiles(&self, idx: usize) -> (Tile, Tile, Tile) {
            if idx >= self.0.len() {
                panic!(
                    "Index {idx} is not in the tile row of length {}",
                    self.0.len()
                );
            }

            let left = if idx > 0 { self.0[idx - 1] } else { Tile::Safe };
            let right = if idx < self.0.len() - 1 {
                self.0[idx + 1]
            } else {
                Tile::Safe
            };

            (left, self.0[idx], right)
        }

        /// Returns the tile in the _next_ row for an index `idx`.
        fn next_row_tile(&self, idx: usize) -> Tile {
            let (l, c, r) = self.neighboring_tiles(idx);

            if (l.is_trap() && c.is_trap() && !r.is_trap())
                || (!l.is_trap() && c.is_trap() && r.is_trap())
                || (l.is_trap() && !c.is_trap() && !r.is_trap())
                || (!l.is_trap() && !c.is_trap() && r.is_trap())
            {
                Tile::Trap
            } else {
                Tile::Safe
            }
        }

        /// Generates the next row of tiles.
        pub fn next_row(&self) -> Self {
            Self(
                (0..self.0.len())
                    .map(|idx| self.next_row_tile(idx))
                    .collect(),
            )
        }
    }

    /// The trapped room as a whole.
    ///
    /// Can be parsed from text input.
    pub struct TrappedRoom {
        /// The number of rows in the room.
        num_rows: usize,
        /// The first row of tiles.
        first_row: TileRow,
    }
    impl Parsable for TrappedRoom {
        type Parsed<'a> = Self;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(
                (
                    field_line_parser("num rows:", nom::character::complete::usize),
                    field_line_parser("first row:", TileRow::parser),
                ),
                |(num_rows, first_row)| Self {
                    num_rows,
                    first_row,
                },
            )
            .parse(input)
        }
    }
    impl TrappedRoom {
        /// Returns an [`Iterator`] over the rows of the room.
        ///
        /// The first yielded item is the `first_row` and the iterator stops
        /// after `num_rows`.
        pub fn generate_rows(&self) -> impl Iterator<Item = TileRow> {
            std::iter::successors(Some(self.first_row.clone()), |row| Some(row.next_row()))
                .take(self.num_rows)
        }

        /// Counts and returns the number of safe tiles in the entire room.
        pub fn count_safe_tiles(&self) -> u64 {
            self.generate_rows()
                .map(|row| row.0.iter().filter_count::<u64>(|t| !t.is_trap()))
                .sum()
        }

        /// Returns another room that is the same but with a different number of
        /// rows.
        pub fn with_num_rows(&self, num_rows: usize) -> Self {
            Self {
                num_rows,
                first_row: self.first_row.clone(),
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 18,
    name: "Like a Rogue",
    preprocessor: Some(|input| Ok(Box::new(TrappedRoom::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<TrappedRoom>()?
                .count_safe_tiles()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<TrappedRoom>()?
                .with_num_rows(400_000)
                .count_safe_tiles()
                .into())
        },
    ],
};
