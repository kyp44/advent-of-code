use std::str::FromStr;

use nom::{
    character::complete::{line_ending, space1},
    combinator::map,
    multi::separated_list1,
};

use crate::aoc::{parse::trim, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(12796), Unsigned(18063)],
    "7,4,9,5,11,17,23,2,0,14,21,24,10,16,13,6,15,25,12,22,18,20,8,19,3,26,1

22 13 17 11  0
 8  2 23  4 24
21  9 14 16  7
 6 10  3 18  5
 1 12 20 15 19

 3 15  0  2 22
 9 18 13 17  5
19  8  7 25 23
20 11 10 24  4
14 21 16 12  6

14 21 17 24  4
10 16 15  9 19
18  8 23 26 20
22 11 13  6  5
 2  0 12  3  7",
    vec![4512u64, 1924].answer_vec()
    }
}

struct BoardCell {
    number: u8,
    hit: bool,
}
impl From<u8> for BoardCell {
    fn from(number: u8) -> Self {
        BoardCell { number, hit: false }
    }
}
impl Parseable<'_> for BoardCell {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        map(nom::character::complete::u8, |v| v.into())(input)
    }
}

struct BingoBoard {
    grid: Grid<BoardCell>,
}
impl Parseable<'_> for BingoBoard {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        let (input, rows) = separated_list1(
            line_ending,
            trim(false, separated_list1(space1, BoardCell::parser)),
        )(input)?;

        Ok((
            input,
            Self {
                grid: Grid::from_data(rows.into_iter().map(|row| row.into_boxed_slice()).collect())
                    .unwrap(),
            },
        ))
    }
}
impl BingoBoard {
    fn call(&mut self, number: u8) -> bool {
        for point in self.grid.all_points() {
            let cell = self.grid.element_at(&point);
            if cell.number == number {
                cell.hit = true;
            }
        }
        self.check_win()
    }

    fn check_win(&self) -> bool {
        // Check rows first
        for row in self.grid.rows_iter() {
            if row.iter().all(|c| c.hit) {
                return true;
            }
        }

        // Check columns
        for col in 0..self.grid.size().x {
            if self.grid.col_iter(col).all(|cell| cell.hit) {
                return true;
            }
        }

        false
    }

    fn score(&self, last_number: u8) -> u64 {
        self.grid
            .all_values()
            .filter_map(|c| {
                if c.hit {
                    None
                } else {
                    Some(u64::from(c.number))
                }
            })
            .sum::<u64>()
            * u64::from(last_number)
    }
}

struct BingoGame {
    calls: Box<[u8]>,
    boards: Box<[BingoBoard]>,
}
impl FromStr for BingoGame {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.split("\n\n");
        let calls = u8::from_csv(lines.next().unwrap())?.into_boxed_slice();
        let boards = BingoBoard::gather(lines)?.into_boxed_slice();

        // Verify boards
        for (board_num, board) in boards.iter().enumerate() {
            if *board.grid.size() != GridSize::new(5, 5) {
                return Err(AocError::InvalidInput(
                    format!("Board {} is not 5 x 5", board_num).into(),
                ));
            }
        }

        Ok(Self { calls, boards })
    }
}
impl BingoGame {
    fn play_until(mut self, num_boards: usize) -> AocResult<u64> {
        let mut boards_won = 0;
        for number in self.calls.iter() {
            for board in self.boards.iter_mut() {
                if !board.check_win() && board.call(*number) {
                    boards_won += 1;
                    if boards_won == num_boards {
                        // We have our final winner!
                        return Ok(board.score(*number));
                    }
                }
            }
        }
        Err(AocError::Process(
            format!("Called numbers ran out before {} board(s) won", num_boards).into(),
        ))
    }

    fn play(self) -> AocResult<u64> {
        self.play_until(1)
    }

    fn play_to_last(self) -> AocResult<u64> {
        let last_board = self.boards.len();
        self.play_until(last_board)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 4,
    name: "Giant Squid",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let game = BingoGame::from_str(input.expect_input()?)?;

            // Process
            Ok(game.play()?.into())
        },
        // Part b)
        |input| {
            // Generation
            let game = BingoGame::from_str(input.expect_input()?)?;

            // Process
            Ok(game.play_to_last()?.into())
        },
    ],
};
