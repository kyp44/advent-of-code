use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    const INPUT: &str = "Player 1 starting position: 4
Player 2 starting position: 8";

    solution_tests! {
        example {
            input = INPUT;
            answers = unsigned![739785];
        }
        expensive_example {
            input = INPUT;
            answers = &[None, Some(Answer::Unsigned(444356092776315))];
        }
        actual_answers = unsigned![864900, 575111835924670];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{
        parse::field_line_parser,
        tree_search::{GlobalStateTreeNode, NodeAction},
    };
    use bare_metal_modulo::{MNum, OffsetNumC};
    use derive_new::new;
    use itertools::Itertools;
    use multiset::HashMultiSet;
    use nom::{combinator::map, sequence::pair};

    /// The winning score needed to end a the game using the deterministic die.
    const DETERMINISTIC_WINNING_SCORE: u32 = 1000;
    /// The winning score needed to end a the game using the Dirac die.
    const DIRAC_WINNING_SCORE: u32 = 21;

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
        /// Rolls the die and returns the rolled value.
        fn roll(&mut self) -> u32 {
            let ret = self.next + 1;
            self.next = ret % 100;
            self.times_rolled += 1;
            ret
        }
    }

    /// The quantum Dirac die used in part two.
    #[derive(new)]
    struct DiracDie;
    impl DiracDie {
        /// Rolls the die some number of times and returns a multi-set of the sums of the rolls.
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
        /// Creates a new player with a particular starting position on the board.
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
    impl Parsable<'_> for Player {
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
        /// Plays the game with the deterministic die and return the loser's score
        /// times the number of rolls.
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
                    if player.score >= DETERMINISTIC_WINNING_SCORE {
                        return game.players[1 - i].score * die.times_rolled;
                    }
                }
            }
        }

        /// Plays the game with Dirac die and return the number of universes in which the winning player wins.
        pub fn play_dirac(&self) -> u64 {
            let state = GameNode::from(self.clone()).traverse_tree(GameGlobalState::default());
            state.num_universes_wins[0].max(state.num_universes_wins[1])
        }
    }

    /// Global state when searching the game tree.
    #[derive(Debug)]
    struct GameGlobalState {
        /// Number of universes in which each player wins.
        num_universes_wins: [u64; 2],
        /// Constant multi set in which the elements are each die roll value, and the number of elements is
        /// the number of universes in which that roll occurs.
        rolls: HashMultiSet<u32>,
    }
    impl Default for GameGlobalState {
        fn default() -> Self {
            Self {
                num_universes_wins: [0; 2],
                rolls: DiracDie::new().roll(NUM_ROLLS_PER_TURN),
            }
        }
    }

    /// A node in the game tree that represents a turn that just happened.
    #[derive(Debug)]
    struct GameNode {
        /// The current state of players.
        game: Game,
        /// The player number that just moved to arrive at this state.
        turn: usize,
        /// The total number of universes in which the current state occurs in this branch.
        num_universes: u64,
    }
    impl GameNode {
        /// Returns whether the previously moved player won.
        fn win(&self) -> bool {
            self.game.players[self.turn].score >= DIRAC_WINNING_SCORE
        }
    }
    impl From<Game> for GameNode {
        fn from(value: Game) -> Self {
            Self {
                game: value,
                turn: 1,
                num_universes: 1,
            }
        }
    }
    impl GlobalStateTreeNode for GameNode {
        type GlobalState = GameGlobalState;

        fn recurse_action(self, global_state: &mut Self::GlobalState) -> NodeAction<Self> {
            if self.win() {
                global_state.num_universes_wins[self.turn] += self.num_universes;
                return NodeAction::Stop;
            }

            NodeAction::Continue(
                global_state
                    .rolls
                    .distinct_elements()
                    .sorted()
                    .map(|r| {
                        let num_universes = u64::try_from(global_state.rolls.count_of(r)).unwrap();
                        let turn = 1 - self.turn;
                        let mut game = self.game.clone();
                        game.players[turn].move_player(*r);

                        Self {
                            game,
                            turn,
                            num_universes: self.num_universes * num_universes,
                        }
                    })
                    .collect(),
            )
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
