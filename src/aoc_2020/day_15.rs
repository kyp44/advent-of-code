use std::convert::TryInto;

use nom::{
    bytes::complete::tag,
    character::complete::{digit1, space0},
    combinator::map,
    multi::separated_list1,
    sequence::tuple,
};

use crate::aoc::{ParseResult, Parseable, Solution};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![],
    "0,3,6",
    vec![Some(436), Some(175594)],
    "1,3,2",
    vec![Some(1), Some(2578)],
    "2,1,3",
    vec![Some(10), Some(3544142)],
    "1,2,3",
    vec![Some(27), Some(261214)],
    "2,3,1",
    vec![Some(78), Some(6895259)],
    "3,2,1",
    vec![Some(438), Some(18)],
    "3,1,2",
    vec![Some(1836), Some(362)]
    }
}

struct Game {
    starting: Vec<u64>,
}

impl Parseable for Game {
    fn parse(input: &str) -> ParseResult<Self> {
        map(
            separated_list1(tuple((space0, tag(","), space0)), digit1),
            |v: Vec<&str>| Game {
                starting: v.into_iter().map(|ds| ds.parse().unwrap()).collect(),
            },
        )(input)
    }
}

impl Game {
    fn play(&self, to_turn: usize) -> u64 {
        let mut turns = self.starting.clone();
        turns.reserve(to_turn.saturating_sub(turns.len()));

        let mut last_turn = *turns.last().unwrap();
        for turn_num in turns.len()..to_turn {
            let last_tn = turns.iter().enumerate().rev().skip(1).find_map(|(tn, v)| {
                if *v == last_turn {
                    Some(tn)
                } else {
                    None
                }
            });
            let next_tn = match last_tn {
                Some(tn) => turn_num - tn - 1,
                None => 0,
            }
            .try_into()
            .unwrap();

            turns.push(next_tn);
            last_turn = next_tn;
            //println!("Turn {}: {}", turn_num + 1, last_turn);
        }
        last_turn
    }
}

pub const SOLUTION: Solution = Solution {
    day: 15,
    name: "Rambunctious Recitation",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let game = Game::from_str(input.trim())?;

            // Process
            Ok(game.play(2020))
        },
        // Part b)
        /*|input| {
            // Generation
            let game = Game::from_str(input.trim())?;

            // Process
            Ok(game.play(30000000))
        },*/
    ],
};
