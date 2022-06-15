use nom::{character::complete::one_of, combinator::map, multi::fill};

use crate::aoc::{parse::trim, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(123)],
    "..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##..##
#..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#......#.###
.######.###.####...#.##.##..#..#..#####.....#.#....###..#.##......#.....#.
.#..#..##..#...##.######.####.####.#.#...#.......#..#.#.#...####.##.#.....
.#..#...##.#.##..#...##.#.##..###.#......#.#.......#.#.#.####.###.##...#..
...####.#..#..#.##.#....##..#.####....##...##..#...#......#.#.......#.....
..##..####..#...#.#.#...##..#.#..###..#####........#..####......#..#

#..#.
#....
##..#
..#..
..###",
    vec![123u64].answer_vec()
    }
}

struct Algorithm {
    table: [bool; 512],
}
impl Parseable<'_> for Algorithm {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        let table = [false; 512];
        map(
            fill(map(trim(one_of(".#")), |c| c == '#'), &mut table),
            |_| Self { table },
        )(input)
    }
}
impl Algorithm {
    fn lookup(&self, value: usize) -> Option<bool> {
        self.table.get(value).copied()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 20,
    name: "Trench Map",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation

            // Process
            Ok(0u64.into())
        },
    ],
};
