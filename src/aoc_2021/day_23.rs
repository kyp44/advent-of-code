use std::{collections::HashSet, fmt, iter::repeat_with};

use enum_map::{enum_map, Enum, EnumMap};
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

#[derive(Debug, Clone, Copy, Enum, EnumIter)]
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
impl fmt::Display for Amphipod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Amphipod::Amber => 'A',
                Amphipod::Bronze => 'B',
                Amphipod::Copper => 'C',
                Amphipod::Desert => 'D',
            }
        )
    }
}
impl Amphipod {
    fn required_energy(&self) -> u32 {
        match self {
            Amphipod::Amber => 1,
            Amphipod::Bronze => 10,
            Amphipod::Copper => 100,
            Amphipod::Desert => 1000,
        }
    }
}

#[derive(Debug, Clone)]
enum SpaceType {
    Hall,
    Side(Amphipod),
}

#[derive(Debug, Clone, Copy, Enum)]
enum RoomSpaceType {
    Adjacent,
    Deep,
}

#[derive(Clone)]
struct Board {
    graph: UnGraph<SpaceType, u8>,
    hall_spaces: Vec<NodeIndex>,
    room_spaces: EnumMap<Amphipod, EnumMap<RoomSpaceType, NodeIndex>>,
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
                use RoomSpaceType::*;
                let rooms = enum_map! {
                    Adjacent => graph.add_node(SpaceType::Side(amph)),
                    Deep => graph.add_node(SpaceType::Side(amph)),
                };

                graph.add_edge(rooms[Adjacent], rooms[Deep], 1);
                graph.add_edge(hall_spaces[i + 1], hall_spaces[i + 2], 2);
                graph.add_edge(hall_spaces[i + 1], rooms[Adjacent], 2);
                graph.add_edge(hall_spaces[i + 2], rooms[Adjacent], 2);

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
            for (rst, space) in self.room_spaces[amph].iter() {
                writeln!(f, "Room space: {:?} {:?}", rst, self.graph[*space])?;
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
static BORDER_DISP: &str = "#";
static EMPTY_DISP: &str = ".";

#[derive(new, PartialEq, Eq)]
struct Position {
    positions: EnumMap<Amphipod, HashSet<NodeIndex>>,
}
impl Parseable<'_> for Position {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        let amphipod_line = move |input| -> NomParseResult<&str, Vec<Amphipod>> {
            terminated(
                trim(
                    false,
                    delimited(
                        many1(tag(BORDER_DISP)),
                        separated_list1(tag(BORDER_DISP), map(one_of("ABCD"), Amphipod::from)),
                        many1(tag(BORDER_DISP)),
                    ),
                ),
                line_ending,
            )(input)
        };

        map(
            delimited(
                tuple((
                    terminated(count(tag(BORDER_DISP), 13), line_ending),
                    terminated(
                        delimited(tag(BORDER_DISP), count(tag("."), 11), tag(BORDER_DISP)),
                        line_ending,
                    ),
                )),
                count(amphipod_line, 2),
                trim(false, count(tag(BORDER_DISP), 9)),
            ),
            |rows| {
                let mut positions: EnumMap<Amphipod, _> = Amphipod::iter()
                    .map(|amph| (amph, HashSet::new()))
                    .collect();

                for (room_amph, (adj_amph, deep_amph)) in
                    Amphipod::iter().zip(rows[0].iter().zip(rows[1].iter()))
                {
                    positions[*adj_amph]
                        .insert(BOARD.room_spaces[room_amph][RoomSpaceType::Adjacent]);
                    positions[*deep_amph].insert(BOARD.room_spaces[room_amph][RoomSpaceType::Deep]);
                }

                Position { positions }
            },
        )(input)
    }
}
impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fmt_spaces =
            |f: &mut fmt::Formatter<'_>, spaces: &[NodeIndex], sep: &str| -> fmt::Result {
                for space in spaces {
                    match self.occupant(space) {
                        Some(a) => write!(f, "{a}")?,
                        None => write!(f, "{EMPTY_DISP}")?,
                    }
                    write!(f, "{sep}")?;
                }
                Ok(())
            };

        // Hall spaces
        writeln!(f, "{}", BORDER_DISP.repeat(13))?;
        write!(f, "{BORDER_DISP}")?;
        fmt_spaces(f, &BOARD.hall_spaces[0..2], "")?;
        write!(f, "{EMPTY_DISP}")?;
        for i in 2..5 {
            fmt_spaces(f, &BOARD.hall_spaces[i..=i], EMPTY_DISP)?;
        }
        fmt_spaces(f, &BOARD.hall_spaces[5..7], "")?;
        writeln!(f, "{BORDER_DISP}")?;

        // Room spaces
        let room_spaces = |space_type| -> Vec<NodeIndex> {
            Amphipod::iter()
                .map(|amph| BOARD.room_spaces[amph][space_type])
                .collect()
        };
        write!(f, "{}", BORDER_DISP.repeat(3))?;
        fmt_spaces(f, &room_spaces(RoomSpaceType::Adjacent), BORDER_DISP)?;
        writeln!(f, "{}", BORDER_DISP.repeat(2))?;
        write!(f, "  {BORDER_DISP}")?;
        fmt_spaces(f, &room_spaces(RoomSpaceType::Deep), BORDER_DISP)?;
        writeln!(f, "  ")?;
        writeln!(f, "  {}  ", BORDER_DISP.repeat(9))?;

        Ok(())
    }
}
impl Position {
    fn occupant(&self, space: &NodeIndex) -> Option<Amphipod> {
        for (amph, idxs) in self.positions.iter() {
            if idxs.contains(space) {
                return Some(amph);
            }
        }
        return None;
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
            let pos = Position::from_str(input.expect_input()?)?;

            println!("{pos}");

            //println!("{:?}", *BOARD);

            // Process
            Ok(0u64.into())
        },
    ],
};
