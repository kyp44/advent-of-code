use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "abc";
            answers = unsigned![22728, 22551];
        }
        actual_answers = unsigned![16106, 22423];
    }
}

/// Contains solution implementation items.
mod solution {
    use derive_more::Display;
    use itertools::Itertools;
    use std::marker::PhantomData;

    /// Trait for the main hash function.
    pub trait HashFunction {
        /// Hashes a string.
        fn hash(s: &str) -> Hash;
    }
    /// The hash function for part one, which is just a single MD5 hash.
    pub enum BasicHash {}
    impl HashFunction for BasicHash {
        fn hash(s: &str) -> Hash {
            Hash(format!("{:x}", md5::compute(s.as_bytes())))
        }
    }
    /// The hash function for part two, which is 17 successive MD5 hash
    /// functions.
    pub enum StretchingHash {}
    impl HashFunction for StretchingHash {
        fn hash(s: &str) -> Hash {
            let mut hash = BasicHash::hash(s);

            for _ in 0..2016 {
                hash = BasicHash::hash(&hash.0);
            }

            hash
        }
    }

    /// A hash string created by a [`HashFunction`] .
    #[derive(Display)]
    #[display("{_0}")]
    pub struct Hash(String);
    impl Hash {
        /// Returns the first character that appears in the hashed string at
        /// least three times in a row, or `None` if no such sequence
        /// was found.
        pub fn any_three(&self) -> Option<char> {
            for (a, b, c) in self.0.chars().tuple_windows() {
                if a == b && b == c {
                    return Some(a);
                }
            }
            None
        }

        /// Returns whether or not a characer `c` was found in the
        /// hashed string at least five times in a row.
        pub fn exact_five(&self, c: char) -> bool {
            self.0.contains(&c.to_string().repeat(5))
        }
    }

    /// A cache that stores a contiguous sequence of [`struct@Hash`]es.
    pub struct HashCache<'a, H> {
        /// The salt string for creating the hashes.
        salt: &'a str,
        /// The hashes in the sequence that have been calculated so far.
        hashes: Vec<Hash>,
        /// Phantom data for the hash function type `H`.
        _phant: PhantomData<H>,
    }
    impl<'a, H: HashFunction> HashCache<'a, H> {
        /// Creates a new cache from a salt string.
        pub fn new(salt: &'a str) -> Self {
            Self {
                salt: salt.trim(),
                hashes: Vec::new(),
                _phant: PhantomData,
            }
        }

        /// Returns the calculated or stored hash for a given index `idx`.
        ///
        /// If the `idx` is outside the calculated sequence, all hashes up to
        /// and including `idx` will be calculated.
        fn get_or_calculate(&mut self, idx: usize) -> &Hash {
            // Calculate if needed
            for idx in self.hashes.len()..=idx {
                self.hashes.push(H::hash(&format!("{}{}", self.salt, idx)));
            }

            &self.hashes[idx]
        }

        /// Returns an [`Iterator`] over the indices of the hashes that qualify
        /// as keys.
        pub fn key_indices(&mut self) -> KeyIndexIter<'_, 'a, H> {
            KeyIndexIter {
                hash_cash: self,
                next_idx: 0,
            }
        }
    }

    /// An [`Iterator`] over the indices of hashes that qualify as valid keys.
    ///
    /// This iterator never terminates.
    pub struct KeyIndexIter<'a, 'b, H> {
        /// The cache used to store the hashes.
        hash_cash: &'a mut HashCache<'b, H>,
        /// The index of the _next_ hash that needs to be checked as a possible
        /// key.
        next_idx: usize,
    }
    impl<H: HashFunction> Iterator for KeyIndexIter<'_, '_, H> {
        type Item = usize;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let curr_idx = self.next_idx;
                let hash = self.hash_cash.get_or_calculate(curr_idx);
                self.next_idx += 1;

                if let Some(c) = hash.any_three() {
                    for idx in self.next_idx..self.next_idx + 1000 {
                        let hash = self.hash_cash.get_or_calculate(idx);
                        if hash.exact_five(c) {
                            return Some(curr_idx);
                        }
                    }
                }
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 14,
    name: "One-Time Pad",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            let mut cache = HashCache::<BasicHash>::new(input.expect_text()?);

            // Process
            Ok(u64::try_from(cache.key_indices().iterations(64).unwrap())
                .unwrap()
                .into())
        },
        // Part two
        |input| {
            let mut cache = HashCache::<StretchingHash>::new(input.expect_text()?);

            // Process
            Ok(u64::try_from(cache.key_indices().iterations(64).unwrap())
                .unwrap()
                .into())
        },
    ],
};
