use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::{expensive_test, solution_test};
    use Answer::Unsigned;

    const INPUT: &str = "Player 1 starting position: 4
Player 2 starting position: 8";

    solution_test! {
    vec![Unsigned(864900), Unsigned(575111835924670)],
    INPUT,
    vec![739785u64].answer_vec()
    }

    expensive_test! {
    INPUT,
    vec![None, Some(Unsigned(444356092776315))]
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::field_line_parser;
    use bare_metal_modulo::{MNum, OffsetNumC};
    use cgmath::{Vector2, Zero};
    use derive_new::new;
    use itertools::Itertools;
    use multiset::HashMultiSet;
    use nom::{combinator::map, sequence::pair};

    /// The deterministic die used in part one.
    #[derive(new)]
    struct DeterministicDie {
        /// The value of the next roll of the die.
        #[new(value = "0")]
        next: u32,
        /// The number of times the die was rolled.
        #[new(value = "0")]
        times_rolled: u32,
    }
    impl DeterministicDie {
        /// Roll the die and return the rolled value.
        fn roll(&mut self) -> u32 {
            let ret = self.next + 1;
            self.next = ret % 100;
            self.times_rolled += 1;
            ret
        }
    }

    /// The quantum Dirac die used in part two.
    #[derive(new)]
    struct DiracDie {}
    impl DiracDie {
        /// Roll the die some number of times and return a multi-set of the sums of the rolls.
        fn roll(&self, num_rolls: usize) -> HashMultiSet<u32> {
            (0..num_rolls)
                .map(|_| 1..=3)
                .multi_cartesian_product()
                .map(|v| v.into_iter().sum::<u32>())
                .collect()
        }
    }

    /// The current state of a player, whose initial position can be parsed from text input.
    #[derive(Debug, Clone)]
    struct Player {
        /// The current position on the board.
        position: OffsetNumC<u32, 10, 1>,
        /// The current score.
        score: u32,
    }
    impl Player {
        /// Create a new player with a particular starting position on the board.
        fn new(position: u32) -> Self {
            Self {
                position: OffsetNumC::new(position),
                score: 0,
            }
        }

        /// Moves a player by a number of spaces.
        fn move_player(&mut self, spaces: u32) {
            self.position += spaces;
            self.score += self.position.a();
        }

        /// Returns the current position of the player on the board.
        fn _position(&self) -> u32 {
            self.position.a()
        }
    }
    impl Parseable<'_> for Player {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                pair(
                    field_line_parser("Player", nom::character::complete::u32),
                    field_line_parser("starting position:", nom::character::complete::u32),
                ),
                |(_, pos)| Self::new(pos),
            )(input.trim())
        }
    }

    /// The number of die rolls per turn.
    const NUM_ROLLS_PER_TURN: usize = 3;

    /// The current state of a game, the initial state of which can be parsed from text input.
    #[derive(Debug, Clone)]
    pub struct Game {
        /// The current state of both players.
        players: [Player; 2],
    }
    impl FromStr for Game {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let players = Player::gather(s.lines())?;
            Ok(Self {
                players: players.try_into().expect("Incorrect number of players"),
            })
        }
    }
    impl Game {
        /// Play the game with the deterministic die and return the loser's score times the number of rolls
        pub fn play_deterministic(&self) -> u32 {
            let mut game = self.clone();
            let mut die = DeterministicDie::new();

            loop {
                for (i, player) in game.players.iter_mut().enumerate() {
                    let roll = (0..NUM_ROLLS_PER_TURN).map(|_| die.roll()).sum();

                    player.move_player(roll);
                    /*println!(
                        "Player {} rolled {} and moved to space {} for a total score of {}",
                        i,
                        roll,
                        player._position(),
                        player.score
                    );*/
                    if player.score >= 1000 {
                        return game.players[1 - i].score * die.times_rolled;
                    }
                }
            }
        }

        /// Play the game with Dirac die and return the number of universes in which the winning player wins.
        pub fn play_dirac(&self) -> u64 {
            let rolls = DiracDie::new().roll(NUM_ROLLS_PER_TURN);

            /// Recursive sub-function of [`Game::play_dirac`] that takes the current game and returns
            /// the number of universes in which each player wins the game.
            fn play_dirac_rec(game: &Game, rolls: &HashMultiSet<u32>, turn: usize) -> Vector2<u64> {
                let mut universes = Vector2::zero();
                for r in rolls.distinct_elements() {
                    let num_universes = u64::try_from(rolls.count_of(r)).unwrap();
                    let mut game = game.clone();
                    let player = &mut game.players[turn];
                    player.move_player(*r);
                    if player.score >= 21 {
                        // This player has won in these universes
                        universes[turn] += num_universes;
                    } else {
                        // Need to recurse
                        universes += num_universes * play_dirac_rec(&game, rolls, 1 - turn);
                    }
                }
                universes
            }

            let universes = play_dirac_rec(self, &rolls, 0);
            universes[0].max(universes[1])
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 21,
    name: "Dirac Dice",
    preprocessor: Some(|input| Ok(Box::new(Game::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input.expect_data::<Game>()?.play_deterministic().into(),
            ))
        },
        // Part two
        |input| {
            // Process
            Ok(input.expect_data::<Game>()?.play_dirac().into())
        },
    ],
};
