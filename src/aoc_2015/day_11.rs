use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "abcdefgh";
            answers = string!["abcdffaa"];
        }
        expensive_example {
            input = "ghijklmn";
            answers = string!["ghjaabcc"];
        }
        actual_answers = string!["hxbxxyzz", "hxcaabcc"];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use std::convert::TryInto;

    /// Increments a character by a certain number in terms of the ASCII code,
    /// which will simply increment the letter for basic letters.
    pub fn char_add(c: char, i: u32) -> char {
        std::char::from_u32((c as u32) + i).unwrap_or(c)
    }

    /// Extension trait to determine if a password is valid.
    trait Password {
        /// Determines if the password is valid according to the security restrictions.
        fn is_valid(&self) -> bool;
    }
    impl Password for str {
        fn is_valid(&self) -> bool {
            /// Disallowed characters according to the security rules.
            const BAD_CHARS: &[char] = &['i', 'o', 'l'];

            // Contains a straight of 3 consecutive letters
            self.chars()
            .collect::<Vec<char>>()
            .windows(3)
            .any(|w| (0..3).all(|i: usize| w[i] == char_add(w[0], i.try_into().unwrap())))
            // Does not contain any forbidden characters
            && BAD_CHARS.iter().all(|c| !self.contains(*c))
            // Two different, non-overlapping pairs
            && self.split_runs().filter_count::<usize>(|s| s.len() > 1) > 1
        }
    }

    /// [`Iterator`] over passwords where each character is incremented by lexical order.
    pub struct LexOrder {
        /// Current string.
        chars: Vec<char>,
    }
    impl LexOrder {
        /// Creates a new [`Iterator`] from a starting string.
        fn new(s: &str) -> Self {
            LexOrder {
                chars: s.chars().rev().collect(),
            }
        }

        /// Returns an [`Iterator`] from a starting string of only valid passwords.
        pub fn valid(s: &str) -> impl Iterator<Item = String> {
            Self::new(s).filter(|s| s.is_valid())
        }
    }
    impl Iterator for LexOrder {
        type Item = String;

        fn next(&mut self) -> Option<Self::Item> {
            if self.chars.is_empty() {
                return None;
            }
            let mut i = 0;
            loop {
                if self.chars[i] == 'z' {
                    if i == self.chars.len() - 1 {
                        return None;
                    }
                    self.chars[i] = 'a';
                    i += 1;
                } else {
                    self.chars[i] = char_add(self.chars[i], 1);
                    break;
                }
            }
            Some(self.chars.iter().rev().collect())
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 11,
    name: "Corporate Policy",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let mut passwords = LexOrder::valid(input.expect_input()?.trim());

            // Process
            Ok(passwords.next().unwrap().into())
        },
        // Part two
        |input| {
            // Generation
            let mut passwords = LexOrder::valid(input.expect_input()?.trim());

            // Process
            Ok(passwords.iterations(2).unwrap().into())
        },
    ],
};
