use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(123)],
    "1163751742
    1381373672
    2136511328
    3694931569
    7463417111
    1319128137
    1359912421
    3125421639
    1293138521
    2311944581",
    vec![123u64].answer_vec()
    }
}

struct RiskLevels {
    grid: Grid<u8>,
}
impl CharGrid<u8> for RiskLevels {
    fn from_char(c: char) -> Option<u8> {
        todo!()
    }

    fn to_char(e: &u8) -> char {
        todo!()
    }

    fn get_grid(&self) -> &Grid<u8> {
        todo!()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 15,
    name: "Chiton",
    solvers: &[
        // Part a)
        |input| {
            // Generation

            // Process
            Ok(0u64.into())
        },
    ],
};
