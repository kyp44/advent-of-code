use std::str::FromStr;

use nom::{
    bytes::complete::tag,
    combinator::map,
    sequence::{pair, preceded},
};

use crate::aoc::{parse::trim, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(864900)],
    "Player 1 starting position: 4
    Player 2 starting position: 8",
    vec![739785u64].answer_vec()
    }
}

#[derive(new)]
struct Die {
    #[new(value = "0")]
    next: u32,
    #[new(value = "0")]
    times_rolled: u32,
}
impl Die {
    fn roll(&mut self) -> u32 {
        let ret = self.next + 1;
        self.next = (self.next + 1) % 100;
        self.times_rolled += 1;
        ret
    }
}

const NUM_SPACES: u32 = 10;

/// Represents a player's position on the board
#[derive(Debug)]
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

const NUM_ROLLS_PER_TURN: u32 = 3;

#[derive(Debug)]
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
    /// Play the game and return the losers score times the number of rolls
    fn play(&mut self) -> u32 {
        let mut die = Die::new();

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
}

pub const SOLUTION: Solution = Solution {
    day: 21,
    name: "Dirac Dice",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let mut game = Game::from_str(input.expect_input()?)?;

            // Process
            Ok(Answer::Unsigned(game.play().into()))
        },
    ],
};
