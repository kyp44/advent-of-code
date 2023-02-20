use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
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

/// Contains solution implementation items.
mod solution {
    use super::*;
    use nom::{
        bytes::complete::tag,
        character::complete::multispace1,
        combinator::map,
        multi::separated_list1,
        sequence::{delimited, pair},
    };
    use std::{collections::HashSet, convert::TryInto, str::FromStr};

    /// The deck of space cards for a player, which can be parsed from text input.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Deck {
        /// Player number (1 or 2).
        player: u8,
        /// Card values in order from bottom to top.
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
        /// Draws the top card of the deck if the deck is not empty.
        fn draw(&mut self) -> Option<u8> {
            self.cards.pop()
        }

        /// Returns whether the deck is exhausted, that is there are no cards left.
        fn done(&self) -> bool {
            self.cards.is_empty()
        }

        /// Places a card onto the bottom of the deck.
        fn place_bottom(&mut self, c: u8) {
            self.cards.insert(0, c);
        }

        /// Calculates the score of the deck.
        pub fn score(&self) -> u64 {
            self.cards
                .iter()
                .enumerate()
                .map(|(x, y)| TryInto::<u64>::try_into(x + 1).unwrap() * Into::<u64>::into(*y))
                .sum()
        }

        /// Returns the number of cards in the deck.
        fn len(&self) -> usize {
            self.cards.len()
        }

        /// Creates a new deck with the specified cards for the same player.
        fn make_new(&self, cards: &[u8]) -> Deck {
            Deck {
                player: self.player,
                cards: cards.to_vec(),
            }
        }
    }

    /// Behavior specific to a particular part of the problem.
    pub trait Part {
        /// Plays the game according to the rules for the part.
        fn play(game: Game) -> Deck;
    }

    /// Behavior for part one.
    pub struct PartOne;
    impl Part for PartOne {
        fn play(mut game: Game) -> Deck {
            loop {
                if game.player1.done() {
                    break game.player2;
                } else if game.player2.done() {
                    break game.player1;
                }
                let c1 = game.player1.draw().unwrap();
                let c2 = game.player2.draw().unwrap();
                if c2 > c1 {
                    game.player2.place_bottom(c2);
                    game.player2.place_bottom(c1);
                } else {
                    // In the event of a draw (which should never happen), player 1 wins
                    game.player1.place_bottom(c1);
                    game.player1.place_bottom(c2);
                }
            }
        }
    }

    /// Behavior for part two.
    pub struct PartTwo;
    impl Part for PartTwo {
        fn play(mut game: Game) -> Deck {
            let mut history = HashSet::new();

            loop {
                //println!("Game: {:?}", game);

                if history.contains(&game) {
                    break game.player1;
                }
                if game.player1.done() {
                    break game.player2;
                } else if game.player2.done() {
                    break game.player1;
                }
                history.insert(game.clone());

                let c1 = game.player1.draw().unwrap();
                let c2 = game.player2.draw().unwrap();
                //println!("Player 1 drew: {}, Player 2 drew: {}", c1, c2);
                let s1 = game.player1.len();
                let s2 = game.player2.len();
                if s1 >= c1.into() && s2 >= c2.into() {
                    //println!("Starting sub-game:");
                    let sub_game = game.make_new(
                        &game.player1.cards[(s1 - Into::<usize>::into(c1))..],
                        &game.player2.cards[(s2 - Into::<usize>::into(c2))..],
                    );
                    if Self::play(sub_game).player == 1 {
                        game.player1.place_bottom(c1);
                        game.player1.place_bottom(c2);
                    } else {
                        game.player2.place_bottom(c2);
                        game.player2.place_bottom(c1);
                    }
                } else if c2 > c1 {
                    game.player2.place_bottom(c2);
                    game.player2.place_bottom(c1);
                } else {
                    // In the event of a draw (which should never happen), player 1 wins
                    game.player1.place_bottom(c1);
                    game.player1.place_bottom(c2);
                }
            }
        }
    }

    /// A game yet to be played, which can be parsed from text input.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Game {
        /// The initial deck for player 1.
        player1: Deck,
        /// The initial deck for player 2.
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

            /// Internal function for [`Game::from_str`].
            ///
            /// Verifies that the player numbers are correct.
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
        /// Makes a new game from two fresh initial decks of cards for each player.
        fn make_new(&self, cards1: &[u8], cards2: &[u8]) -> Game {
            Game {
                player1: self.player1.make_new(cards1),
                player2: self.player2.make_new(cards2),
            }
        }

        /// Play the game according to the rules for the part and returns
        /// the ending deck state for the winning player.
        pub fn play<P: Part>(&self) -> Deck {
            P::play(self.clone())
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 22,
    name: "Crab Combat",
    preprocessor: Some(|input| Ok(Box::new(input.parse::<Game>()?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Game>()?
                .play::<PartOne>()
                .score()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Game>()?
                .play::<PartTwo>()
                .score()
                .into())
        },
    ],
};
