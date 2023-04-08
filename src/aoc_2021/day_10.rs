use aoc::prelude::*;
use itertools::Itertools;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "[({(<(())[]>[[{[]{<()<>>
[(()[<>])]({[<{<<[]>>(
{([(<{}[<>[]}>{[]{[(<()>
(((({<>}<{<{<>}{[]{[]{}
[[<[([]))<([[{}[[()]]]
[{[{({}]{}}([{[{{{}}([]
{<[[]]>}<{[{[{[]{()[[[]
[<(<(<(<{}))><([]([]()
<{([([[(<>()){}]>(<<{{
<{([{{}}[<[[[<>{}]]]>[]]";
            answers = vec![26397u64, 288957].answer_vec();
        }
        actual_answers = vec![Unsigned(266301), Unsigned(3404870164)];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use derive_new::new;
    use nom::{
        character::complete::one_of,
        combinator::{all_consuming, map},
        multi::many1,
    };

    /// The parity of a chunk symbol.
    enum ChunkParity {
        /// Open, which is the start of a new chunk.
        Open,
        /// Close, which closes out a chunk.
        Close,
    }

    /// The type of a chunk.
    #[derive(PartialEq, Eq, Clone, Copy)]
    pub enum ChunkType {
        /// Chunk denoted with parentheses.
        Paren,
        /// Chunk denoted with square brackets.
        Square,
        /// Chunk denotes with curly braces.
        Brace,
        /// Chunk denoted with angle brackets.
        Angle,
    }
    impl ChunkType {
        /// Returns the score of the chunk for syntax checkers.
        pub fn score_corrupted(&self) -> u64 {
            match *self {
                ChunkType::Paren => 3,
                ChunkType::Square => 57,
                ChunkType::Brace => 1197,
                ChunkType::Angle => 25137,
            }
        }

        /// Returns the score of the chunk for auto completers.
        fn score_incomplete(&self) -> u64 {
            match *self {
                ChunkType::Paren => 1,
                ChunkType::Square => 2,
                ChunkType::Brace => 3,
                ChunkType::Angle => 4,
            }
        }
    }

    /// A chunk symbol, which can be parsed from text input.
    #[derive(new)]
    struct ChunkSymbol {
        /// Type of the symbol.
        chunk_type: ChunkType,
        /// The parity of the symbol.
        parity: ChunkParity,
    }
    impl Parseable<'_> for ChunkSymbol {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(one_of("()[]{}<>"), |c| {
                use ChunkParity::*;
                use ChunkType::*;

                match c {
                    '(' => ChunkSymbol::new(Paren, Open),
                    ')' => ChunkSymbol::new(Paren, Close),
                    '[' => ChunkSymbol::new(Square, Open),
                    ']' => ChunkSymbol::new(Square, Close),
                    '{' => ChunkSymbol::new(Brace, Open),
                    '}' => ChunkSymbol::new(Brace, Close),
                    '<' => ChunkSymbol::new(Angle, Open),
                    '>' => ChunkSymbol::new(Angle, Close),
                    _ => panic!(),
                }
            })(input)
        }
    }

    /// The status of a line once analyzed.
    pub enum LineStatus {
        /// A valid line with no problems.
        Valid,
        /// A corrupted line with the first illegal chunk symbol.
        Corrupted(ChunkType),
        /// Incomplete with the ordered symbol pattern needed to complete it.
        Incomplete(Vec<ChunkType>),
    }
    impl LineStatus {
        /// Returns the score for the line given its status.
        ///
        /// Valid lines have no score.
        pub fn score(&self) -> Option<u64> {
            match self {
                LineStatus::Valid => None,
                LineStatus::Corrupted(chunk) => Some(chunk.score_corrupted()),
                LineStatus::Incomplete(seq) => {
                    Some(seq.iter().fold(0, |a, ct| 5 * a + ct.score_incomplete()))
                }
            }
        }
    }

    /// A Line of chunks, which can be parsed from text input.
    pub struct Line {
        /// The ordered list of chunk symbols.
        chunks: Box<[ChunkSymbol]>,
    }
    impl Parseable<'_> for Line {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                all_consuming(trim(false, many1(ChunkSymbol::parser))),
                |chunks| Self {
                    chunks: chunks.into_boxed_slice(),
                },
            )(input)
        }
    }
    impl Line {
        /// Analyzes the line and return its status.
        pub fn analyze(&self) -> LineStatus {
            let mut stack = Vec::new();
            for chunk in self.chunks.iter() {
                match chunk.parity {
                    ChunkParity::Open => stack.push(chunk.chunk_type),
                    ChunkParity::Close => {
                        let status = LineStatus::Corrupted(chunk.chunk_type);
                        match stack.pop() {
                            Some(oct) => {
                                if oct != chunk.chunk_type {
                                    return status;
                                }
                            }
                            None => {
                                return status;
                            }
                        }
                    }
                }
            }
            if stack.is_empty() {
                LineStatus::Valid
            } else {
                stack.reverse();
                LineStatus::Incomplete(stack)
            }
        }
    }
}

use num::Integer;
use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 10,
    name: "Syntax Scoring",
    preprocessor: Some(|input| {
        Ok(Box::new(
            input
                .lines()
                .map(|line| Ok(Line::from_str(line)?.analyze()))
                .collect::<AocResult<Vec<_>>>()?,
        )
        .into())
    }),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Vec<LineStatus>>()?
                .iter()
                .filter_map(|ls| {
                    if matches!(ls, LineStatus::Corrupted(_)) {
                        Some(ls.score().unwrap())
                    } else {
                        None
                    }
                })
                .sum::<u64>()
                .into())
        },
        // Part two
        |input| {
            // Process
            let scores = input
                .expect_data::<Vec<LineStatus>>()?
                .iter()
                .filter_map(|ls| {
                    if matches!(ls, LineStatus::Incomplete(_)) {
                        Some(ls.score().unwrap())
                    } else {
                        None
                    }
                })
                .sorted()
                .collect::<Vec<_>>();
            if scores.len().is_even() {
                return Err(AocError::Process(
                    "One of the incomplete lines is missing an even number of closers!".into(),
                ));
            }
            Ok(scores[(scores.len() - 1) / 2].into())
        },
    ],
};
