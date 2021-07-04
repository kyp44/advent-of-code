use std::{collections::HashMap, convert::TryInto};

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
    vec![1428],
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
        // Maps the spoken number to the last turn number
        let mut turn_map: HashMap<u64, u64> = self
            .starting
            .iter()
            .take(self.starting.len() - 1)
            .enumerate()
            .map(|(t, s)| (*s, t.try_into().unwrap()))
            .collect();

        let mut last_spoken = *self.starting.last().unwrap();
        for turn in self.starting.len()..to_turn {
            let turn: u64 = turn.try_into().unwrap();
            let next_spoken = match turn_map.get(&last_spoken) {
                Some(t) => turn - t - 1,
                None => 0,
            };
            turn_map.insert(last_spoken, turn - 1);
            last_spoken = next_spoken;
            //println!("Turn {}: {}", turn + 1, last_spoken);
            //println!("Turn map: {:?}", turn_map);
        }
        last_spoken
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
