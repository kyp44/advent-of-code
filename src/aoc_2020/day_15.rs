use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "0,3,6";
            answers = vec![436u64, 175594].answer_vec();
        }
        expensive_example {
            input = "1,3,2";
            answers = vec![1u64, 2578].answer_vec();
        }
        expensive_example {
            input = "2,1,3";
            answers = vec![10u64, 3544142].answer_vec();
        }
        expensive_example {
            input = "1,2,3";
            answers = vec![27u64, 261214].answer_vec();
        }
        expensive_example {
            input = "2,3,1";
            answers = vec![78u64, 6895259].answer_vec();
        }
        expensive_example {
            input = "3,2,1";
            answers = vec![438u64, 18].answer_vec();
        }
        expensive_example {
            input = "3,1,2";
            answers = vec![1836u64, 362].answer_vec();
        }
        actual_answers = vec![Unsigned(1428), Unsigned(3718541)];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use nom::{
        bytes::complete::tag, character::complete::space0, combinator::map, multi::separated_list1,
        sequence::tuple,
    };
    use std::convert::TryInto;

    /// A memory game played by the elves, which can be parsed from text input.
    pub struct Game {
        /// List of starting numbers.
        starting: Vec<u64>,
    }
    impl Parseable<'_> for Game {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_list1(
                    tuple((space0, tag(","), space0)),
                    nom::character::complete::u64,
                ),
                |starting| Game { starting },
            )(input)
        }
    }
    impl Game {
        /// Plays the game up to some number of turns, returning the number spoken at that turn.
        pub fn play(&self, to_turn: usize) -> u64 {
            // Maps the spoken number to the last turn number
            // This had been implemented before as a HashMap but was
            // pretty slow in debug mode, so we traded memory usage
            // for speed and use a potentially very large vector instead
            // to avoid the time penalty of HashMap lookups.
            let mut turn_map: Vec<Option<u64>> = vec![None; to_turn];

            // Initialize with starting numbers
            for (t, s) in self
                .starting
                .iter()
                .take(self.starting.len() - 1)
                .enumerate()
            {
                let s: usize = (*s).try_into().unwrap();
                turn_map[s] = Some(t.try_into().unwrap());
            }

            // Now play the rest of the game.
            let mut last_spoken = *self.starting.last().unwrap();
            for turn in self.starting.len()..to_turn {
                let turn: u64 = turn.try_into().unwrap();
                let ls: usize = last_spoken.try_into().unwrap();
                let next_spoken = match turn_map[ls] {
                    Some(t) => turn - t - 1,
                    None => 0,
                };
                turn_map[ls] = Some(turn - 1);
                last_spoken = next_spoken;
                /*println!("Turn {}: {}", turn + 1, last_spoken);
                if last_spoken == 0 {
                    let mut keys: Vec<&u64> = turn_map.keys().collect();
                    keys.sort_unstable();
                    println!("Turn map: {:?}", keys);
                }*/
            }
            last_spoken
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 15,
    name: "Rambunctious Recitation",
    preprocessor: Some(|input| Ok(Box::new(Game::from_str(input.trim())?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<Game>()?.play(2020).into())
        },
        // Part two
        |input| {
            // Process
            Ok(input.expect_data::<Game>()?.play(30000000).into())
        },
    ],
};
