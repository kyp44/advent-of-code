use std::{convert::TryInto, str::FromStr};

use nom::{
    bytes::complete::tag,
    character::complete::{digit1, multispace1},
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, pair},
};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Number;

    solution_test! {
    vec![Number(33561)],
    "Player 1:
9
2
6
3
1

Player 2:
5
8
4
7
10",
    vec![306].answer_vec()
    }
}

#[derive(Debug)]
struct Deck {
    player: u8,
    cards: Vec<u8>,
}
impl Parseable<'_> for Deck {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(
            pair(
                delimited(tag("Player "), digit1, pair(tag(":"), multispace1)),
                separated_list1(multispace1, u8::parser),
            ),
            |(ps, mut v)| {
                v.reverse();
                Deck {
                    player: ps.parse().unwrap(),
                    cards: v,
                }
            },
        )(input)
    }
}
impl Deck {
    fn draw(&mut self) -> Option<u8> {
        self.cards.pop()
    }

    fn done(&self) -> bool {
        self.cards.is_empty()
    }

    fn place_bottom(&mut self, c: u8) {
        self.cards.insert(0, c);
    }

    fn score(&self) -> u64 {
        self.cards
            .iter()
            .enumerate()
            .map(|(x, y)| TryInto::<u64>::try_into(x + 1).unwrap() * Into::<u64>::into(*y))
            .sum()
    }
}

#[derive(Debug)]
struct Game {
    player1: Deck,
    player2: Deck,
}
impl FromStr for Game {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let secs = s.sections(2)?;
        let game = Game {
            player1: Deck::from_str(secs[0])?,
            player2: Deck::from_str(secs[1])?,
        };

        // Check that the player numbers are correct
        fn check(exp: u8, found: u8) -> AocResult<()> {
            match found {
                x if x == exp => Ok(()),
                _ => Err(AocError::InvalidInput(format!(
                    "Expected player {} deck, found player {}",
                    exp, found
                ))),
            }
        }
        check(1, game.player1.player)?;
        check(2, game.player2.player)?;

        Ok(game)
    }
}
impl Game {
    fn play(&mut self) -> u64 {
        let winning_cards = loop {
            if self.player1.done() {
                break &self.player2;
            } else if self.player2.done() {
                break &self.player1;
            }
            let c1 = self.player1.draw().unwrap();
            let c2 = self.player2.draw().unwrap();
            if c2 > c1 {
                self.player2.place_bottom(c2);
                self.player2.place_bottom(c1);
            } else {
                // In the event of a draw (which should never happen), player 1 wins
                self.player1.place_bottom(c1);
                self.player1.place_bottom(c2);
            }
        };
        winning_cards.score()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 22,
    name: "Crab Combat",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let mut game: Game = input.parse()?;

            // Process
            Ok(game.play().into())
        },
    ],
};
