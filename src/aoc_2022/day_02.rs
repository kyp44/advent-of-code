use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
    example {
    input = "A Y
B X
C Z";
    answers = vec![15u64, 12].answer_vec();
    }
    actual_answers = vec![Unsigned(13809), Unsigned(12316)];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use nom::{
        character::complete::{anychar, space1},
        combinator::map,
        error::ParseError,
        sequence::separated_pair,
    };

    /// The outcome of a round of the game.
    #[derive(Clone, Copy)]
    enum Outcome {
        /// A win 😀.
        Win,
        /// A loss 🙁.
        Lose,
        /// A draw 😐.
        Draw,
    }
    impl Outcome {
        /// Determines the score value for this outcome.
        pub fn score(&self) -> u8 {
            match self {
                Outcome::Win => 6,
                Outcome::Lose => 0,
                Outcome::Draw => 3,
            }
        }
    }
    impl From<HandShape> for Outcome {
        fn from(value: HandShape) -> Self {
            match value {
                HandShape::Rock => Self::Lose,
                HandShape::Paper => Self::Draw,
                HandShape::Scissors => Self::Win,
            }
        }
    }

    /// A hand shape.
    #[derive(Clone, Copy)]
    enum HandShape {
        /// Rock.
        Rock,
        /// Paper.
        Paper,
        /// Scissors.
        Scissors,
    }
    impl Parseable<'_> for HandShape {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            let (out, c) = anychar(input)?;
            Ok((
                out,
                match c {
                    'A' | 'X' => HandShape::Rock,
                    'B' | 'Y' => HandShape::Paper,
                    'C' | 'Z' => HandShape::Scissors,
                    _ => {
                        return Err(nom::Err::Error(NomParseError::from_error_kind(
                            input,
                            nom::error::ErrorKind::IsNot,
                        )));
                    }
                },
            ))
        }
    }
    impl HandShape {
        /// Determines the score when the you choose this shape.
        pub fn score(&self) -> u8 {
            match self {
                HandShape::Rock => 1,
                HandShape::Paper => 2,
                HandShape::Scissors => 3,
            }
        }

        /// Returns the numerical value for this shape.
        fn value(&self) -> u8 {
            match self {
                HandShape::Rock => 0,
                HandShape::Paper => 1,
                HandShape::Scissors => 2,
            }
        }

        /// Returns the outcome if this shape plays another.
        pub fn beats(&self, other: Self) -> Outcome {
            let (a, b) = (self.value(), other.value());
            if a == b {
                Outcome::Draw
            } else if (b + 1) % 3 == a {
                Outcome::Win
            } else {
                Outcome::Lose
            }
        }

        /// Returns the shape by you for you to get the `outcome` against this shape.
        pub fn needed(&self, outcome: Outcome) -> Self {
            match self {
                HandShape::Rock => match outcome {
                    Outcome::Win => HandShape::Paper,
                    Outcome::Lose => HandShape::Scissors,
                    Outcome::Draw => HandShape::Rock,
                },
                HandShape::Paper => match outcome {
                    Outcome::Win => HandShape::Scissors,
                    Outcome::Lose => HandShape::Rock,
                    Outcome::Draw => HandShape::Paper,
                },
                HandShape::Scissors => match outcome {
                    Outcome::Win => HandShape::Rock,
                    Outcome::Lose => HandShape::Paper,
                    Outcome::Draw => HandShape::Scissors,
                },
            }
        }
    }

    /// A round of Rock, Paper, Scissors.
    pub struct Round {
        /// Your opponent's shape.
        other_shape: HandShape,
        /// Your shape (or the [`Outcome`] for part two).
        your_shape: HandShape,
    }
    impl Parseable<'_> for Round {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(HandShape::parser, space1, HandShape::parser),
                |pair| Self {
                    other_shape: pair.0,
                    your_shape: pair.1,
                },
            )(input)
        }
    }
    impl Round {
        /// Determines the score using the (incorrect) part one interpretation.
        pub fn score(&self) -> u8 {
            self.your_shape.score() + self.your_shape.beats(self.other_shape).score()
        }

        /// Determines the score using the (correct) part two interpretation.
        pub fn score_correct(&self) -> u8 {
            let outcome: Outcome = self.your_shape.into();
            let your_shape = self.other_shape.needed(outcome);

            your_shape.score() + outcome.score()
        }
    }

    /// An overall game of Rock, Paper, Scissors, consisting of many rounds.
    pub struct Game {
        /// The rounds that this game comprises.
        rounds: Vec<Round>,
    }
    impl FromStr for Game {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                rounds: Round::gather(s.lines())?,
            })
        }
    }
    impl Game {
        /// Determines the total score given the score function for a [`Round`].
        pub fn score(&self, score_func: fn(&Round) -> u8) -> u64 {
            self.rounds
                .iter()
                .map(|r| u64::from(score_func(r)))
                .sum::<u64>()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 2,
    name: "Rock Paper Scissors",
    preprocessor: Some(|input| Ok(Box::new(Game::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<Game>()?.score(Round::score).into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Game>()?
                .score(Round::score_correct)
                .into())
        },
    ],
};
