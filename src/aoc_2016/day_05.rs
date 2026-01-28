use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "abc";
            answers = string!["18f47a30", "05ace8e3"];
        }
        actual_answers = string!["801b56a7", "424a0197"];
    }
}

/// Contains solution implementation items.
mod solution {
    use md5::Digest;
    use std::marker::PhantomData;

    /// A trait for the doors encountered in the problem.
    pub trait Door {
        /// The type of password clue extracted from hashes.
        type PasswordClue;

        /// Returns the password clue for an [`Md5Hash`] if everything
        /// is valid, and `None` otherwise.
        fn password_clue(hash: &Md5Hash) -> Option<Self::PasswordClue>;

        /// Finds the password for this door given the `door_id`.
        fn find_password(door_id: &str) -> String;
    }

    /// The first door from part one.
    ///
    /// Here the clue is simply the next [`char`] in the password.
    pub enum DoorOne {}
    impl Door for DoorOne {
        type PasswordClue = char;

        fn password_clue(hash: &Md5Hash) -> Option<Self::PasswordClue> {
            hash.is_valid()
                .then(|| char::from_digit(u32::from(hash.0.0[2]) & 0x0F, 16).unwrap())
        }

        fn find_password(door_id: &str) -> String {
            PasswordClues::<Self>::new(door_id).take(8).collect()
        }
    }

    /// A clue for the second door in part two.
    pub struct DoorTwoPasswordClue {
        /// The position of the `char` in the password.
        position: usize,
        /// The character in the `position`.
        char: char,
    }

    /// The second door from part two.
    ///
    /// Here the clue is a [`DoorTwoPasswordClue`].
    pub enum DoorTwo {}
    impl Door for DoorTwo {
        type PasswordClue = DoorTwoPasswordClue;

        fn password_clue(hash: &Md5Hash) -> Option<Self::PasswordClue> {
            let position: usize = (hash.0.0[2] & 0x0F).into();

            (hash.is_valid() && position < 8).then(|| DoorTwoPasswordClue {
                position,
                char: char::from_digit(u32::from(hash.0.0[3]) >> 4, 16).unwrap(),
            })
        }

        fn find_password(door_id: &str) -> String {
            let mut password = [None; 8];
            for pc in PasswordClues::<Self>::new(door_id) {
                if password[pc.position].is_none() {
                    password[pc.position] = Some(pc.char);

                    // Have we solved the whole password?
                    if password.iter().all(|c| c.is_some()) {
                        break;
                    }
                }
            }

            password.into_iter().map(|c| c.unwrap()).collect()
        }
    }

    /// An MD5 hash.
    pub struct Md5Hash(Digest);
    impl Md5Hash {
        /// Computes the hash from a `door_id` and an `index`.
        pub fn compute(door_id: &str, index: u64) -> Self {
            Self(md5::compute(format!("{}{}", door_id, index)))
        }

        /// Returns whether or not the hash itself is valid.
        ///
        /// Note that this does take into account whether the clue is valid.
        pub fn is_valid(&self) -> bool {
            let hd = &self.0.0;

            // Test whether the first 5 hex digits are zero
            hd[0] == 0 && hd[1] == 0 && hd[2] < 16
        }
    }

    /// An [`Iterator`] over _valid_ password clues for a [`Door`].
    struct PasswordClues<'a, D: Door> {
        /// The door ID.
        door_id: &'a str,
        /// The index for the next hash.
        index: u64,
        /// Just a phantom for the [`Door`] type `D`.
        _phant: PhantomData<D>,
    }
    impl<'a, D: Door> PasswordClues<'a, D> {
        /// Creates a new [`Iterator`] starting at index 0.
        pub fn new(door_id: &'a str) -> Self {
            Self {
                door_id,
                index: 0,
                _phant: Default::default(),
            }
        }
    }
    impl<D: Door> Iterator for PasswordClues<'_, D> {
        type Item = D::PasswordClue;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let hash = Md5Hash::compute(self.door_id, self.index);
                self.index += 1;
                if let Some(pc) = D::password_clue(&hash) {
                    break Some(pc);
                }
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 5,
    name: "How About a Nice Game of Chess?",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(DoorOne::find_password(input.expect_text()?.trim()).into())
        },
        // Part two
        |input| {
            // Process
            Ok(DoorTwo::find_password(input.expect_text()?.trim()).into())
        },
    ],
};
