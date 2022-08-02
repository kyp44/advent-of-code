use std::{
    collections::{HashMap, HashSet},
    fmt,
    iter::repeat_with,
};

use nom::{
    bytes::complete::tag,
    character::complete::{line_ending, one_of},
    combinator::map,
    multi::{count, many1, separated_list1},
    sequence::{delimited, terminated, tuple},
};
use petgraph::{graph::NodeIndex, prelude::UnGraph};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::aoc::{parse::trim, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(123)],
    "#############
#...........#
###B#C#A#D###
#B#C#D#A#
#########",
    vec![123u64].answer_vec()
    }
}

#[derive(Debug, Clone, Copy, EnumIter, PartialEq, Eq, Hash)]
enum Amphipod {
    Amber,
    Bronze,
    Copper,
    Desert,
}
impl From<char> for Amphipod {
    fn from(c: char) -> Self {
        match c {
            'A' => Amphipod::Amber,
            'B' => Amphipod::Bronze,
            'C' => Amphipod::Copper,
            _ => Amphipod::Desert,
        }
    }
}

#[derive(Debug, Clone)]
enum SpaceType {
    Hall,
    Side(Amphipod),
}

#[derive(Clone)]
struct Board {
    graph: UnGraph<SpaceType, u8>,
    hall_spaces: Vec<NodeIndex>,
    room_spaces: HashMap<Amphipod, Vec<NodeIndex>>,
}
impl Board {
    fn new() -> Self {
        let mut graph = UnGraph::new_undirected();

        // All the hall spaces
        let hall_spaces: Vec<_> = repeat_with(|| graph.add_node(SpaceType::Hall))
            .take(7)
            .collect();

        // Connect the end hall spaces
        graph.add_edge(hall_spaces[0], hall_spaces[1], 1);
        graph.add_edge(hall_spaces[5], hall_spaces[6], 1);

        // Build and connect the side rooms
        let room_spaces = Amphipod::iter()
            .enumerate()
            .map(|(i, amph)| {
                let rooms = vec![
                    graph.add_node(SpaceType::Side(amph)),
                    graph.add_node(SpaceType::Side(amph)),
                ];

                graph.add_edge(rooms[0], rooms[1], 1);
                graph.add_edge(hall_spaces[i + 1], hall_spaces[i + 2], 2);
                graph.add_edge(hall_spaces[i + 1], rooms[1], 2);
                graph.add_edge(hall_spaces[i + 2], rooms[1], 2);

                (amph, rooms)
            })
            .collect();

        Self {
            graph,
            hall_spaces,
            room_spaces,
        }
    }
}
impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for space in self.hall_spaces.iter() {
            writeln!(f, "Hall space: {:?} {:?}", space, self.graph[*space])?;
        }
        for amph in Amphipod::iter() {
            for space in self.room_spaces[&amph].iter() {
                writeln!(f, "Room space: {:?} {:?}", space, self.graph[*space])?;
            }
        }
        for edge in self.graph.raw_edges() {
            writeln!(
                f,
                "Edge: from {:?} to {:?}: {:?}",
                edge.source(),
                edge.target(),
                edge.weight
            )?;
        }
        Ok(())
    }
}

lazy_static! {
    static ref BOARD: Board = Board::new();
}

#[derive(new)]
struct Position {
    positions: HashMap<Amphipod, HashSet<NodeIndex>>,
}
impl Parseable<'_> for Position {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        let border = "#";

        let amphipod_line = move |input| -> NomParseResult<&str, Vec<Amphipod>> {
            terminated(
                trim(
                    false,
                    delimited(
                        many1(tag(border)),
                        separated_list1(tag(border), map(one_of("ABCD"), Amphipod::from)),
                        many1(tag(border)),
                    ),
                ),
                line_ending,
            )(input)
        };

        map(
            delimited(
                tuple((
                    terminated(count(tag(border), 13), line_ending),
                    terminated(
                        delimited(tag(border), count(tag("."), 11), tag(border)),
                        line_ending,
                    ),
                )),
                count(amphipod_line, 2),
                trim(false, count(tag(border), 9)),
            ),
            |_v| Position {
                positions: HashMap::new(),
            },
        )(input)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 23,
    name: "Amphipod",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let _pos = Position::from_str(input.expect_input()?)?;

            //println!("{:?}", *BOARD);

            // Process
            Ok(0u64.into())
        },
    ],
};
