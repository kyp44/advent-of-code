use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "ULL
RRDDD
LURDL
UUUUD";
            answers = string!["1985", "5DB3"];
        }
        actual_answers = string!["69642", "8CB23"];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{grid::Digit, parse::trim};
    use euclid::Vector2D;
    use itertools::join;
    use nom::{branch::alt, bytes::complete::tag, combinator::map, multi::many1};

    /// A direction to move on a keypad.
    ///
    /// Can be parsed from text input.
    #[derive(Clone, Copy)]
    pub enum Direction {
        /// Up (negative `y`).
        Up,
        /// Down (positive `y`).
        Down,
        /// Left (negative `x`).
        Left,
        /// Right (positive `x`).
        Right,
    }
    impl Parsable for Direction {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            alt((
                map(tag("U"), |_| Self::Up),
                map(tag("D"), |_| Self::Down),
                map(tag("L"), |_| Self::Left),
                map(tag("R"), |_| Self::Right),
            ))
            .parse(input)
        }
    }
    impl Direction {
        /// Returns the unit vector corresponding with this direction.
        pub fn as_vector(&self) -> Vector2D<isize, GridSpace> {
            match self {
                Self::Up => -Vector2D::<isize, _>::unit_y(),
                Self::Down => Vector2D::unit_y(),
                Self::Left => -Vector2D::<isize, _>::unit_x(),
                Self::Right => Vector2D::unit_x(),
            }
        }
    }

    /// Trait for keypads with different layouts.
    pub trait Keypad: Sized {
        /// Returns the [`GridPoint`] if the `point` is a valid point on the
        /// keypad and `None` otherwise.
        fn valid_point(&self, point: &AnyGridPoint) -> Option<GridPoint>;
        /// Returns the starting [`Key`] for determine the code.
        fn starting_key(&self) -> Key<'_, Self>;
        /// Returns a key label.
        ///
        /// The `point` is assumed to be valid.
        fn get_key_label(&self, point: &GridPoint) -> char;
    }

    /// The keypad that was pictured in part one.
    ///
    /// This has the following layout:
    /// ```text
    /// 1 2 3
    /// 4 5 6
    /// 7 8 9
    /// ```
    pub struct PicturedKeypad {
        /// The keyboard layout.
        grid: Grid<Digit>,
    }
    impl Default for PicturedKeypad {
        fn default() -> Self {
            Self {
                grid: Grid::from_str(
                    "123
456
789",
                )
                .unwrap(),
            }
        }
    }
    impl Keypad for PicturedKeypad {
        fn valid_point(&self, point: &AnyGridPoint) -> Option<GridPoint> {
            self.grid.bounded_point(point)
        }

        fn starting_key(&self) -> Key<'_, Self> {
            Key {
                pad: self,
                // The 5 key
                point: GridPoint::new(1, 1),
            }
        }

        fn get_key_label(&self, point: &GridPoint) -> char {
            char::from_digit(self.grid.get(point).0.into(), 10).unwrap()
        }
    }

    /// The [`Grid`] space type for the [`ActualKeypad`] of part two.
    #[derive(Clone, Copy)]
    enum ActualKeySpace {
        /// A space that is not a key.
        Invalid,
        /// A space that is a key with its label.
        Valid(char),
    }
    impl TryFrom<char> for ActualKeySpace {
        type Error = ();

        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                '-' => Ok(Self::Invalid),
                _ => {
                    if value.is_ascii_alphanumeric() {
                        Ok(Self::Valid(value))
                    } else {
                        Err(())
                    }
                }
            }
        }
    }
    impl From<ActualKeySpace> for char {
        fn from(value: ActualKeySpace) -> Self {
            match value {
                ActualKeySpace::Invalid => '-',
                ActualKeySpace::Valid(c) => c,
            }
        }
    }

    /// The actual keypad encountered in part two.
    ///
    /// This has the following layout:
    /// ```text
    ///     1
    ///   2 3 4
    /// 5 6 7 8 9
    ///   A B C
    ///     D
    /// ```
    pub struct ActualKeypad {
        /// The keyboard layout.
        grid: Grid<ActualKeySpace>,
    }
    impl Default for ActualKeypad {
        fn default() -> Self {
            Self {
                grid: Grid::from_str(
                    "--1--
-234-
56789
-ABC-
--D--",
                )
                .unwrap(),
            }
        }
    }
    impl Keypad for ActualKeypad {
        fn valid_point(&self, point: &AnyGridPoint) -> Option<GridPoint> {
            self.grid
                .bounded_point(point)
                .and_then(|p| match self.grid.get(&p) {
                    ActualKeySpace::Invalid => None,
                    ActualKeySpace::Valid(_) => Some(p),
                })
        }

        fn starting_key(&self) -> Key<'_, Self> {
            Key {
                pad: self,
                // The 5 key
                point: GridPoint::new(0, 2),
            }
        }

        fn get_key_label(&self, point: &GridPoint) -> char {
            (*self.grid.get(point)).into()
        }
    }

    /// A single key on a [`Keypad`].
    pub struct Key<'a, K: Keypad> {
        /// The [`Keypad`] in which this s a key.
        pad: &'a K,
        /// The location of this key on the keypad.
        point: GridPoint,
    }
    impl<K: Keypad> Clone for Key<'_, K> {
        // Note that we cannot derive this because it stupidly requires that `K: Clone`
        // even though references are always [`Clone`].
        fn clone(&self) -> Self {
            Self {
                pad: self.pad,
                point: self.point,
            }
        }
    }
    impl<K: Keypad> Key<'_, K> {
        /// Returns the label for the key.
        pub fn label(&self) -> char {
            self.pad.get_key_label(&self.point)
        }

        /// Returns the adjacent key in the specified `direction`.
        pub fn adjacent_key(&self, direction: Direction) -> Self {
            let point: AnyGridPoint = self.point.cast();
            let point = point + direction.as_vector();

            Self {
                pad: self.pad,
                point: self.pad.valid_point(&point).unwrap_or(self.point),
            }
        }
    }

    /// A set of moves to moves to go from from key to adjacent key.
    ///
    /// Can be parsed from text input.
    struct MoveSet {
        /// The sequence of directions to move.
        directions: Vec<Direction>,
    }
    impl Parsable for MoveSet {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(trim(false, many1(Direction::parser)), |directions| Self {
                directions,
            })
            .parse(input)
        }
    }
    impl MoveSet {
        /// Applies this move set to a starting [`Key`] and returns the final
        /// [`Key`].
        pub fn apply<'a, K: Keypad>(&self, starting_key: Key<'a, K>) -> Key<'a, K> {
            let mut key = starting_key;
            for dir in self.directions.iter() {
                key = key.adjacent_key(*dir);
            }
            key
        }
    }

    /// Finds the code from multiple move sets applied in succession.
    ///
    /// Can be parsed from text input.
    pub struct CodeFinder {
        /// The sequence of move sets.
        move_sets: Vec<MoveSet>,
    }
    impl FromStr for CodeFinder {
        type Err = NomParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                move_sets: MoveSet::gather(s.lines())?,
            })
        }
    }
    impl CodeFinder {
        /// Finds the code for a particular [`Keypad`].
        pub fn find_code<K: Keypad>(&self, keypad: K) -> String {
            let mut keys = Vec::new();

            self.move_sets
                .iter()
                .fold(keypad.starting_key(), |key, move_set| {
                    let new_key = move_set.apply(key);
                    keys.push(new_key.clone());
                    new_key
                });

            join(keys.into_iter().map(|key| key.label()), "")
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 2,
    name: "Bathroom Security",
    preprocessor: Some(|input| Ok(Box::new(CodeFinder::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<CodeFinder>()?
                .find_code(PicturedKeypad::default())
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<CodeFinder>()?
                .find_code(ActualKeypad::default())
                .into())
        },
    ],
};
