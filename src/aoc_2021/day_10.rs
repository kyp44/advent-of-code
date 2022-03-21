use itertools::Itertools;
use nom::{
    character::complete::one_of,
    combinator::{all_consuming, map},
    multi::many1,
};

use crate::aoc::{parse::trim, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(266301), Unsigned(3404870164)],
    "[({(<(())[]>[[{[]{<()<>>
[(()[<>])]({[<{<<[]>>(
{([(<{}[<>[]}>{[]{[(<()>
(((({<>}<{<{<>}{[]{[]{}
[[<[([]))<([[{}[[()]]]
[{[{({}]{}}([{[{{{}}([]
{<[[]]>}<{[{[{[]{()[[[]
[<(<(<(<{}))><([]([]()
<{([([[(<>()){}]>(<<{{
<{([{{}}[<[[[<>{}]]]>[]]",
    vec![26397u64, 288957].answer_vec()
    }
}

enum Parity {
    Open,
    Close,
}
#[derive(PartialEq, Eq, Clone, Copy)]
enum ChunkType {
    Paren,
    Square,
    Brace,
    Angle,
}
impl ChunkType {
    fn corrupted_score(&self) -> u64 {
        match *self {
            ChunkType::Paren => 3,
            ChunkType::Square => 57,
            ChunkType::Brace => 1197,
            ChunkType::Angle => 25137,
        }
    }

    fn incomplete_score(&self) -> u64 {
        match *self {
            ChunkType::Paren => 1,
            ChunkType::Square => 2,
            ChunkType::Brace => 3,
            ChunkType::Angle => 4,
        }
    }
}

struct Chunk {
    chunk_type: ChunkType,
    parity: Parity,
}
impl Chunk {
    fn new(chunk_type: ChunkType, parity: Parity) -> Self {
        Chunk { chunk_type, parity }
    }
}
impl Parseable<'_> for Chunk {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(one_of("()[]{}<>"), |c| {
            use ChunkType::*;
            use Parity::*;

            match c {
                '(' => Chunk::new(Paren, Open),
                ')' => Chunk::new(Paren, Close),
                '[' => Chunk::new(Square, Open),
                ']' => Chunk::new(Square, Close),
                '{' => Chunk::new(Brace, Open),
                '}' => Chunk::new(Brace, Close),
                '<' => Chunk::new(Angle, Open),
                '>' => Chunk::new(Angle, Close),
                _ => panic!(),
            }
        })(input)
    }
}

enum LineStatus {
    Valid,
    Corrupted(ChunkType),
    Incomplete(Vec<ChunkType>),
}

struct Line {
    chunks: Box<[Chunk]>,
}
impl Parseable<'_> for Line {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(all_consuming(trim(many1(Chunk::parser))), |chunks| Self {
            chunks: chunks.into_boxed_slice(),
        })(input)
    }
}
impl Line {
    fn analyze(&self) -> LineStatus {
        let mut stack = Vec::new();
        for chunk in self.chunks.iter() {
            match chunk.parity {
                Parity::Open => stack.push(chunk.chunk_type),
                Parity::Close => {
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
            LineStatus::Incomplete(stack)
        }
    }

    fn incomplete_score(&self) -> Option<u64> {
        match self.analyze() {
            LineStatus::Incomplete(stack) => Some(
                stack
                    .into_iter()
                    .rev()
                    .fold(0, |a, ct| 5 * a + ct.incomplete_score()),
            ),
            _ => None,
        }
    }
}

pub const SOLUTION: Solution = Solution {
    day: 10,
    name: "Syntax Scoring",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let lines = Line::gather(input.lines())?;

            // Process
            Ok(lines
                .iter()
                .filter_map(|line| {
                    if let LineStatus::Corrupted(chunk_type) = line.analyze() {
                        Some(chunk_type.corrupted_score())
                    } else {
                        None
                    }
                })
                .sum::<u64>()
                .into())
        },
        // Part b)
        |input| {
            // Generation
            let lines = Line::gather(input.lines())?;

            let scores: Vec<u64> = lines
                .into_iter()
                .filter_map(|line| line.incomplete_score())
                .sorted()
                .collect();
            let len = scores.len();

            if len % 2 == 0 {
                return Err(AocError::Process(
                    "One of the incomplete lines is missing an even number of closers!".into(),
                ));
            }

            // Process
            Ok(scores[(len - 1) / 2].into())
        },
    ],
};
