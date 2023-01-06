use std::str::FromStr;

use cgmath::{Vector2, Zero};
use itertools::Itertools;
use multiset::HashMultiSet;
use nom::{
    bytes::complete::tag,
    combinator::map,
    sequence::{pair, preceded},
};

use crate::aoc::{parse::trim, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expensive_test, solution_test};
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

#[derive(new)]
struct DeterministicDie {
    #[new(value = "0")]
    next: u32,
    #[new(value = "0")]
    times_rolled: u32,
}
impl DeterministicDie {
    fn roll(&mut self) -> u32 {
        let ret = self.next + 1;
        self.next = (self.next + 1) % 100;
        self.times_rolled += 1;
        ret
    }
}

fn dirac_roll(num_rolls: usize) -> HashMultiSet<u32> {
    (0..num_rolls)
        .map(|_| 1..=3)
        .multi_cartesian_product()
        .map(|v| v.into_iter().sum::<u32>())
        .collect()
}

const NUM_SPACES: u32 = 10;

/// Represents a player's position on the board
#[derive(Debug, Clone)]
struct Player {
    position: u32,
    score: u32,
}
impl Player {
    fn new(position: u32) -> Self {
        Self {
            position: (position + NUM_SPACES - 1) % NUM_SPACES,
            score: 0,
        }
    }

    /// Moves a player by the number of spaces
    fn move_player(&mut self, spaces: u32) {
        self.position = (self.position + spaces) % NUM_SPACES;
        self.score += self.position + 1;
    }

    fn _position(&self) -> u32 {
        self.position + 1
    }
}
impl Parseable<'_> for Player {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        fn preceded_u32<'a>(
            text: &'static str,
        ) -> impl FnMut(&'a str) -> NomParseResult<&'a str, u32> {
            preceded(trim(false, tag(text)), nom::character::complete::u32)
        }
        map(
            pair(preceded_u32("Player"), preceded_u32("starting position:")),
            |(_, pos)| Self::new(pos),
        )(input)
    }
}

const NUM_ROLLS_PER_TURN: usize = 3;

#[derive(Debug, Clone)]
struct Game {
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
    fn play_deterministic(&mut self) -> u32 {
        let mut die = DeterministicDie::new();

        loop {
            for (i, player) in self.players.iter_mut().enumerate() {
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
                    return self.players[1 - i].score * die.times_rolled;
                }
            }
        }
    }

    /// Play the game with Dirac die and return the number of universes in which the winning player wins.
    fn play_dirac(&self) -> u64 {
        let rolls = dirac_roll(NUM_ROLLS_PER_TURN);

        /// Recursive version that takes the current game and returns the number of universes in which each
        /// player wins the game.
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

pub const SOLUTION: Solution = Solution {
    day: 21,
    name: "Dirac Dice",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let mut game = Game::from_str(input.expect_input()?)?;

            // Process
            Ok(Answer::Unsigned(game.play_deterministic().into()))
        },
        // Part two
        |input| {
            // Generation
            let game = Game::from_str(input.expect_input()?)?;

            // Process
            Ok(game.play_dirac().into())
        },
    ],
};
