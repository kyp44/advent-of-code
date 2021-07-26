use std::{collections::HashMap, str::FromStr};

use nom::{
    bytes::complete::tag,
    character::complete::{digit1, line_ending, one_of},
    combinator::map,
    multi::{many1, separated_list1},
    sequence::{delimited, pair},
    Finish,
};

use crate::aoc::{AocError, DiscardInput, ParseError, ParseResult, Parseable, Solution};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![],
    "Tile 2311:
..##.#..#.
##..#.....
#...##..#.
####.#...#
##.##.###.
##...#.###
.#.#.#..##
..#....#..
###...#.#.
..###..###

Tile 1951:
#.##...##.
#.####...#
.....#..##
#...######
.##.#....#
.###.#####
###.##.##.
.###....#.
..#.#..#.#
#...##.#..

Tile 1171:
####...##.
#..##.#..#
##.#..#.#.
.###.####.
..###.####
.##....##.
.#...####.
#.##.####.
####..#...
.....##...

Tile 1427:
###.##.#..
.#..#.##..
.#.##.#..#
#.#.#.##.#
....#...##
...##..##.
...#.#####
.#.####.#.
..#..###.#
..##.#..#.

Tile 1489:
##.#.#....
..##...#..
.##..##...
..#...#...
#####...#.
#..#.#.#.#
...#.#.#..
##.#...##.
..##.##.##
###.##.#..

Tile 2473:
#....####.
#..#.##...
#.##..#...
######.#.#
.#...#.#.#
.#########
.###.#..#.
########.#
##...##.#.
..###.#.#.

Tile 2971:
..#.#....#
#...###...
#.#.###...
##.##..#..
.#####..##
.#..####.#
#..#.#..#.
..####.###
..#.#.###.
...#.#.#.#

Tile 2729:
...#.#.#.#
####.#....
..#.#.....
....#..#.#
.##..##.#.
.#.####...
####.#.#..
##.####...
##..#.##..
#.##...##.

Tile 3079:
#.#.#####.
.#..######
..#.......
######....
####.#..#.
.#...#.##.
#.#####.##
..#.###...
..#.......
..#.###...",
    vec![]
    }
}

enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

enum Rotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

struct Tile {
    id: u64,
    edges: HashMap<Edge, Vec<bool>>,
    edges_reversed: HashMap<Edge, Vec<bool>>,
}
impl FromStr for Tile {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (id, lv) = map::<_, _, _, ParseError, _, _>(
            pair(
                delimited(tag("Tile "), digit1, pair(tag(":"), line_ending)),
                separated_list1(line_ending, many1(one_of(".#"))),
            ),
            |(ids, lv): (&str, Vec<Vec<char>>)| (ids.parse().unwrap(), lv),
        )(s)
        .finish()
        .discard_input()?;

        // Verify the tile dimensions
        let size = lv[0].len();
        if lv.len() != size {
            return Err(AocError::InvalidInput(format!(
                "Tile {} did not have the expected height of {}",
                id, size
            )));
        }
        if lv.iter().any(|v| v.len() != size) {
            return Err(AocError::InvalidInput(format!(
                "Not all rows of tile {} have the expected width of {}",
                id, size
            )));
        }

        Ok(Tile {
            id,
            edges: HashMap::new(),
            edges_reversed: HashMap::new(),
        })
    }
}

pub const SOLUTION: Solution = Solution {
    day: 20,
    name: "Jurassic Jigsaw",
    solvers: &[
        // Part a)
        |input| {
            // Generation

            // Process
            Ok(0)
        },
    ],
};
