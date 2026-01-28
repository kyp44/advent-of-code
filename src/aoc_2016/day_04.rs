use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "aaaaa-bbb-z-y-x-123[abxyz]
a-b-c-d-e-f-g-h-987[abcde]
not-a-real-room-404[oarel]
totally-real-room-200[decoy]";
            answers = unsigned![1514];
        }
        actual_answers = unsigned![409147, 991];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use derive_new::new;
    use multiset::HashMultiSet;
    use nom::{
        bytes::complete::tag,
        character::complete::{alpha1, anychar},
        combinator::map,
        multi::{many_m_n, separated_list1},
        sequence::delimited,
    };

    /// A single character in the name or a room, with the frequency that it appears.
    ///
    /// These are comparable in descending order of frequency followed by normal
    /// alphabetical order.
    #[derive(new, PartialEq, Eq)]
    struct NameChar {
        /// The character.
        pub char: char,
        /// The number of times the character appears in the name.
        pub frequency: usize,
    }
    impl Ord for NameChar {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            use std::cmp::Ordering;

            let freq_ord = self.frequency.cmp(&other.frequency);
            match freq_ord {
                Ordering::Equal => {
                    // Now compare chars in normal alphabetical order
                    self.char.cmp(&other.char)
                }
                // Want to reverse this so higher frequency chars come first.
                _ => freq_ord.reverse(),
            }
        }
    }
    impl PartialOrd for NameChar {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    /// A room, which may or may not be real.
    ///
    /// Can be parsed from text input.
    #[derive(Debug, Clone)]
    pub struct Room {
        /// The name of the room, including the hyphens that separate words.
        pub name: String,
        /// The sector ID of the room.
        pub sector_id: u32,
        /// The checksum, guaranteed to have 5 characters.
        checksum: String,
    }
    impl Parsable<'_> for Room {
        fn parser(input: &'_ str) -> NomParseResult<&'_ str, Self> {
            const SEP: &str = "-";
            map(
                trim(
                    false,
                    (
                        separated_list1(tag(SEP), alpha1),
                        tag(SEP),
                        nom::character::complete::u32,
                        delimited(tag("["), many_m_n(5, 5, anychar), tag("]")),
                    ),
                ),
                |(names, _, sector_id, checksum_chars)| Self {
                    name: itertools::join(names, "-"),
                    sector_id,
                    checksum: checksum_chars.into_iter().collect(),
                },
            )
            .parse(input)
        }
    }
    impl Room {
        /// Builds the list of [`NameChar`]s for the name.
        ///
        /// This is not necessarily sorted, and will only contain the alphabetic
        /// characters.
        fn build_name_chars(&self) -> Vec<NameChar> {
            // Add all characters to a multi set
            let chars =
                HashMultiSet::from_iter(self.name.chars().filter(|c| c.is_ascii_alphabetic()));

            // Build the vec from the multiset
            chars
                .distinct_elements()
                .map(|c| NameChar::new(*c, chars.count_of(c)))
                .collect()
        }

        /// Returns whether or not the room is a real room (`true`) or a decoy (`false`).
        pub fn is_real(&self) -> bool {
            // Sort in with higher frequency chars first then alphabetical
            let mut name_chars = self.build_name_chars();
            name_chars.sort();

            self.checksum
                .chars()
                .zip(name_chars.into_iter().map(|nc| nc.char))
                .all(|(check_c, name_c)| check_c == name_c)
        }

        /// Decrypts the name, which is done whether or not the room is real.
        pub fn decrypt(&self) -> String {
            const LOWER_A: u32 = 'a' as u32;
            self.name
                .chars()
                .map(|c| {
                    if c.is_ascii_alphabetic() {
                        char::from_u32(
                            (c.to_ascii_lowercase() as u32 - LOWER_A + self.sector_id) % 26
                                + LOWER_A,
                        )
                        .unwrap()
                    } else {
                        ' '
                    }
                })
                .collect()
        }
    }

    /// A list of [`Room`]s.
    ///
    /// Can be parsed from text input.
    pub struct Rooms {
        /// The rooms.
        rooms: Vec<Room>,
    }
    impl FromStr for Rooms {
        type Err = NomParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                rooms: Room::gather(s.lines())?,
            })
        }
    }
    impl Rooms {
        /// Returns an [`Iterator`] over all the [`Room`]s.
        pub fn iter(&self) -> impl Iterator<Item = &Room> {
            self.rooms.iter()
        }
    }
    impl IntoIterator for Rooms {
        type Item = Room;
        type IntoIter = <Vec<Room> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.rooms.into_iter()
        }
    }
    impl FromIterator<Room> for Rooms {
        fn from_iter<T: IntoIterator<Item = Room>>(iter: T) -> Self {
            Self {
                rooms: iter.into_iter().collect(),
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 4,
    name: "Security Through Obscurity",
    preprocessor: Some(|input| {
        // Go ahead and filter out the fake rooms
        Ok(Box::new(Rooms::from_iter(
            Rooms::from_str(input)?.into_iter().filter(|r| r.is_real()),
        ))
        .into())
    }),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(u64::from(
                input
                    .expect_data::<Rooms>()?
                    .iter()
                    .map(|r| r.sector_id)
                    .sum::<u32>(),
            )
            .into())
        },
        // Part two
        |input| {
            // Generate
            let rooms = input.expect_data::<Rooms>()?;

            // Process
            /* println!("Real rooms decrypted and sector ID:");
            for room in rooms.iter() {
                println!("{}: {}", room.decrypt(), room.sector_id)
            }
            println!(); */

            Ok(u64::from(
                rooms
                    .iter()
                    .find(|r| r.decrypt().contains("north"))
                    .ok_or(AocError::NoSolution)?
                    .sector_id,
            )
            .into())
        },
    ],
};
