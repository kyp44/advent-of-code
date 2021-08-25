use std::convert::TryInto;

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expensive_test, solution_test};
    use Answer::String;

    solution_test! {
    vec![String("hxbxxyzz".into()), String("hxcaabcc".into())],
    "abcdefgh",
    vec!["abcdffaa"].answer_vec()
    }

    expensive_test! {
    "ghijklmn",
    vec!["ghjaabcc"].answer_vec()
    }
}

trait Password {
    fn is_valid(&self) -> bool;
}
impl Password for str {
    fn is_valid(&self) -> bool {
        const BAD_CHARS: &[char] = &['i', 'o', 'l'];

        // Contains a straight of 3 consecutive letters
        self.chars()
            .collect::<Vec<char>>()
            .windows(3)
            .any(|w| (0..3).all(|i: usize| w[i] == char_add(w[0], i.try_into().unwrap())))
        // Does not contain any forbidden characters
        && BAD_CHARS.iter().all(|c| !self.contains(*c))
        // Two different, non-overlapping pairs
        && FilterCount::<_, usize>::filter_count(self.split_runs().map(|s| s.len()), |n| *n > 1) > 1
    }
}

struct LexOrder {
    chars: Vec<char>,
}
impl LexOrder {
    fn new(s: &str) -> Self {
        LexOrder {
            chars: s.chars().rev().collect(),
        }
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

fn valid_iter(s: &str) -> impl Iterator<Item = String> {
    LexOrder::new(s).filter(|s| s.is_valid())
}

pub const SOLUTION: Solution = Solution {
    day: 11,
    name: "Corporate Policy",
    solvers: &[
        // Part a)
        |input| {
            let new_pw = valid_iter(input.trim()).next().unwrap();
            //println!("{}", new_pw);
            Ok(Answer::String(new_pw))
        },
        // Part b)
        |input| {
            let new_pw = valid_iter(input.trim()).nth(1).unwrap();
            Ok(Answer::String(new_pw))
        },
    ],
};
