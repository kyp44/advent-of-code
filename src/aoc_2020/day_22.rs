use std::{collections::HashSet, convert::TryInto, str::FromStr};

use nom::{
    bytes::complete::tag,
    character::complete::multispace1,
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, pair},
};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(33561), Unsigned(34594)],
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
    vec![306u64, 291].answer_vec(),
    "Player 1:
43
19

Player 2:
2
29
14",
    vec![None, Some(Unsigned(105))]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Deck {
    player: u8,
    cards: Vec<u8>,
}
impl Parseable<'_> for Deck {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        map(
            pair(
                delimited(
                    tag("Player "),
                    nom::character::complete::u8,
                    pair(tag(":"), multispace1),
                ),
                separated_list1(multispace1, u8::parser),
            ),
            |(player, mut cards)| {
                cards.reverse();
                Deck { player, cards }
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

    fn len(&self) -> usize {
        self.cards.len()
    }

    fn make_new(&self, cards: &[u8]) -> Deck {
        Deck {
            player: self.player,
            cards: cards.to_vec(),
        }
    }
}

trait Part {}
struct PartOne;
impl Part for PartOne {}
struct PartTwo;
impl Part for PartTwo {}

trait GamePart<P: Part> {
    fn play(&mut self) -> &Deck;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
                _ => Err(AocError::InvalidInput(
                    format!("Expected player {exp} deck, found player {found}").into(),
                )),
            }
        }
        check(1, game.player1.player)?;
        check(2, game.player2.player)?;

        Ok(game)
    }
}
impl Game {
    fn make_new(&self, cards1: &[u8], cards2: &[u8]) -> Game {
        Game {
            player1: self.player1.make_new(cards1),
            player2: self.player2.make_new(cards2),
        }
    }
}
impl GamePart<PartOne> for Game {
    fn play(&mut self) -> &Deck {
        loop {
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
        }
    }
}
impl GamePart<PartTwo> for Game {
    fn play(&mut self) -> &Deck {
        let mut history = HashSet::new();

        loop {
            //println!("Game: {:?}", self);

            if history.contains(self) {
                break &self.player1;
            }
            if self.player1.done() {
                break &self.player2;
            } else if self.player2.done() {
                break &self.player1;
            }
            history.insert(self.clone());

            let c1 = self.player1.draw().unwrap();
            let c2 = self.player2.draw().unwrap();
            //println!("Player 1 drew: {}, Player 2 drew: {}", c1, c2);
            let s1 = self.player1.len();
            let s2 = self.player2.len();
            if s1 >= c1.into() && s2 >= c2.into() {
                //println!("Starting sub-game:");
                let mut sub_game = self.make_new(
                    &self.player1.cards[(s1 - Into::<usize>::into(c1))..],
                    &self.player2.cards[(s2 - Into::<usize>::into(c2))..],
                );
                if GamePart::<PartTwo>::play(&mut sub_game).player == 1 {
                    self.player1.place_bottom(c1);
                    self.player1.place_bottom(c2);
                } else {
                    self.player2.place_bottom(c2);
                    self.player2.place_bottom(c1);
                }
            } else if c2 > c1 {
                self.player2.place_bottom(c2);
                self.player2.place_bottom(c1);
            } else {
                // In the event of a draw (which should never happen), player 1 wins
                self.player1.place_bottom(c1);
                self.player1.place_bottom(c2);
            }
        }
    }
}

pub const SOLUTION: Solution = Solution {
    day: 22,
    name: "Crab Combat",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let mut game: Game = input.expect_input()?.parse()?;

            // Process
            Ok(GamePart::<PartOne>::play(&mut game).score().into())
        },
        // Part two
        |input| {
            // Generation
            let mut game: Game = input.expect_input()?.parse()?;

            // Process
            Ok(GamePart::<PartTwo>::play(&mut game).score().into())
        },
    ],
};
