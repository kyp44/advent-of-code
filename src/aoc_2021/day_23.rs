use std::{cmp::Ordering, fmt, iter::repeat_with, ops::Add};

use enum_map::{enum_map, Enum, EnumMap};
use nom::{
    bytes::complete::tag,
    character::complete::{line_ending, one_of},
    combinator::map,
    multi::{count, many1, separated_list1},
    sequence::{delimited, terminated, tuple},
};
use petgraph::{
    algo::{bellman_ford, FloatMeasure},
    graph::NodeIndex,
    prelude::StableUnGraph,
};
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
###B#C#B#D###
#A#D#C#A#
#########",
    vec![12521u64].answer_vec()
    }
}

#[derive(Debug, Clone, Copy, Enum, EnumIter, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, Enum, PartialEq, Eq)]
enum RoomSpaceType {
    Adjacent,
    Deep,
}

#[derive(Debug, Clone)]
enum SpaceType {
    Hall,
    Room(Amphipod, RoomSpaceType),
}

/// Integer measure for graph distances
#[derive(Clone, Copy, Debug, PartialEq)]
enum Distance {
    Finite(u8),
    Infinite,
}
impl From<u8> for Distance {
    fn from(v: u8) -> Self {
        Self::Finite(v)
    }
}
impl Default for Distance {
    fn default() -> Self {
        Self::Finite(0)
    }
}
impl PartialOrd for Distance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Distance::Finite(a) => match other {
                Distance::Finite(b) => a.partial_cmp(b),
                Distance::Infinite => Some(Ordering::Less),
            },
            Distance::Infinite => match other {
                Distance::Finite(_) => Some(Ordering::Greater),
                Distance::Infinite => None,
            },
        }
    }
}
impl Add for Distance {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Distance::Finite(a) => match rhs {
                Distance::Finite(b) => Self::Finite(a + b),
                Distance::Infinite => Self::Infinite,
            },
            Distance::Infinite => Self::Infinite,
        }
    }
}
impl FloatMeasure for Distance {
    fn zero() -> Self {
        Self::Finite(0)
    }

    fn infinite() -> Self {
        Self::Infinite
    }
}

type Graph = StableUnGraph<SpaceType, Distance>;

#[derive(Clone)]
struct Board {
    graph: Graph,
    hall_spaces: Vec<NodeIndex>,
    room_spaces: EnumMap<Amphipod, EnumMap<RoomSpaceType, NodeIndex>>,
}
impl Board {
    fn new() -> Self {
        let mut graph = Graph::with_capacity(15, 18);

        // All the hall spaces
        let hall_spaces: Vec<_> = repeat_with(|| graph.add_node(SpaceType::Hall))
            .take(7)
            .collect();

        // Connect the end hall spaces
        graph.add_edge(hall_spaces[0], hall_spaces[1], 1.into());
        graph.add_edge(hall_spaces[5], hall_spaces[6], 1.into());

        // Build and connect the side rooms
        let room_spaces = Amphipod::iter()
            .enumerate()
            .map(|(i, amph)| {
                use RoomSpaceType::*;
                let rooms = enum_map! {
                    Adjacent => graph.add_node(SpaceType::Room(amph, Adjacent)),
                    Deep => graph.add_node(SpaceType::Room(amph, Deep)),
                };

                graph.add_edge(rooms[Adjacent], rooms[Deep], 1.into());
                graph.add_edge(hall_spaces[i + 1], hall_spaces[i + 2], 2.into());
                graph.add_edge(hall_spaces[i + 1], rooms[Adjacent], 2.into());
                graph.add_edge(hall_spaces[i + 2], rooms[Adjacent], 2.into());

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
                writeln!(
                    f,
                    "Room space: {:?} {:?} {:?}",
                    space, rst, self.graph[*space]
                )?;
            }
        }
        for edge in self.graph.edge_indices() {
            let end_points = self.graph.edge_endpoints(edge).unwrap();
            writeln!(
                f,
                "Edge: from {:?} to {:?}: {:?}",
                end_points.0,
                end_points.1,
                self.graph.edge_weight(edge)
            )?;
        }
        Ok(())
    }
}

lazy_static! {
    static ref BOARD: Board = Board::new();
}
const BORDER_DISP: &str = "#";
const EMPTY_DISP: &str = ".";

#[derive(Debug)]
struct Move {
    energy: u32,
    new_position: Position,
}

#[derive(new, PartialEq, Eq, Clone)]
struct Position {
    positions: EnumMap<Amphipod, Vec<NodeIndex>>,
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
                let mut positions: EnumMap<Amphipod, _> =
                    Amphipod::iter().map(|amph| (amph, Vec::new())).collect();

                for (room_amph, (adj_amph, deep_amph)) in
                    Amphipod::iter().zip(rows[0].iter().zip(rows[1].iter()))
                {
                    positions[*adj_amph]
                        .push(BOARD.room_spaces[room_amph][RoomSpaceType::Adjacent]);
                    positions[*deep_amph].push(BOARD.room_spaces[room_amph][RoomSpaceType::Deep]);
                }

                Position { positions }
            },
        )(input)
    }
}
impl fmt::Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(self, f)
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

    fn occupied_spaces(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        self.positions.values().flat_map(|ns| ns).copied()
    }

    fn move_amphipod(&mut self, amphipod: Amphipod, old: &NodeIndex, new: NodeIndex) {
        let idx = self.positions[amphipod]
            .iter()
            .position(|n| n == old)
            .unwrap();
        self.positions[amphipod][idx] = new;
    }

    fn moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        // Go through all amphipods at all spaces
        for amphipod in Amphipod::iter() {
            let home_adjacent = &BOARD.room_spaces[amphipod][RoomSpaceType::Adjacent];
            let home_deep = &BOARD.room_spaces[amphipod][RoomSpaceType::Deep];
            let home_deep_occupant = self.occupant(home_deep);

            for space in self.positions[amphipod].iter() {
                let space_type = BOARD.graph.node_weight(*space).unwrap();
                // If we are already home then we do not want to move
                if let SpaceType::Room(a, rst) = space_type {
                    if *a == amphipod && *rst == RoomSpaceType::Deep {
                        continue;
                    }
                    if let Some(da) = self.occupant(&BOARD.room_spaces[*a][RoomSpaceType::Deep]) && *rst == RoomSpaceType::Adjacent && da == amphipod {
                        continue;
                    }
                }

                // Remove all occupied nodes except this one
                let mut graph = BOARD.clone().graph;
                self.occupied_spaces().for_each(|n| {
                    if n != *space {
                        graph.remove_node(n);
                    }
                });

                // Also remove all rooms that we do not want to enter
                for amph in Amphipod::iter() {
                    if match space_type {
                        SpaceType::Hall => amph != amphipod,
                        SpaceType::Room(a, rst) => {
                            if amph == *a {
                                if amph == amphipod {
                                    false
                                } else {
                                    *rst == RoomSpaceType::Adjacent
                                }
                            } else {
                                amph != amphipod
                            }
                        }
                    } {
                        // Remove this entire room (unless we are in it)
                        BOARD.room_spaces[amph].values().for_each(|n| {
                            if n != space {
                                graph.remove_node(*n);
                            }
                        })
                    }
                }

                // Determine shortest paths to all possible destinations
                // TODO
                /* println!("Amph {} at {}", amphipod, space.index());
                               if amphipod == Amphipod::Amber && space.index() == 14 {
                                   dbg!(&graph);
                               }
                */
                let paths = bellman_ford(&graph, *space).unwrap();
                for (distance, node) in graph.node_indices().filter_map(|n| {
                    // Is the node ourself or unreachable?
                    let d = match paths.distances[n.index()] {
                        Distance::Finite(d) => d,
                        Distance::Infinite => return None,
                    };
                    if d == 0 {
                        return None;
                    }

                    // Is this a hall node and we are in the hall?
                    if matches!(graph.node_weight(*space).unwrap(), SpaceType::Hall)
                        && matches!(graph.node_weight(n).unwrap(), SpaceType::Hall)
                    {
                        return None;
                    }

                    // We don't want to move into the home adjacent space unless the deep space has our own kind in it
                    if n == *home_adjacent {
                        match home_deep_occupant {
                            Some(a) => {
                                if a != amphipod {
                                    return None;
                                }
                            }
                            None => return None,
                        }
                    }

                    // If we're in a non-home deep space, we don't want to move into the adjacent space
                    if let SpaceType::Room(a, rst) = space_type && *rst == RoomSpaceType::Deep {
                        if let SpaceType::Room(na, nrst) = graph.node_weight(n).unwrap() && na == a && *nrst == RoomSpaceType::Adjacent {
                            return None;
                        }
                    }

                    Some((d, n))
                }) {
                    // Copy current position and make the move
                    let mut new_position = self.clone();
                    new_position.move_amphipod(amphipod, space, node);

                    // TODO
                    //println!("{:?}", &new_position);
                    moves.push(Move {
                        energy: amphipod.required_energy() * u32::from(distance),
                        new_position,
                    })
                }
            }
        }

        moves
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
            let mut pos = Position::from_str(input.expect_input()?)?;

            println!("Initial:\n{pos}");

            // Manually just make best first move
            pos.move_amphipod(Amphipod::Bronze, &NodeIndex::new(11), NodeIndex::new(2));
            pos.move_amphipod(Amphipod::Copper, &NodeIndex::new(9), NodeIndex::new(11));
            pos.move_amphipod(Amphipod::Desert, &NodeIndex::new(10), NodeIndex::new(3));
            pos.move_amphipod(Amphipod::Bronze, &NodeIndex::new(2), NodeIndex::new(10));
            pos.move_amphipod(Amphipod::Bronze, &NodeIndex::new(7), NodeIndex::new(9));
            pos.move_amphipod(Amphipod::Desert, &NodeIndex::new(13), NodeIndex::new(4));
            pos.move_amphipod(Amphipod::Amber, &NodeIndex::new(14), NodeIndex::new(5));
            pos.move_amphipod(Amphipod::Desert, &NodeIndex::new(4), NodeIndex::new(14));
            pos.move_amphipod(Amphipod::Desert, &NodeIndex::new(3), NodeIndex::new(13));
            pos = pos.moves()[0].new_position.clone();

            // TODO: Next we nee to check foor a solved position!

            println!("Next:\n{pos}");

            // TODO
            for (_i, pos) in pos.moves().into_iter().enumerate() {
                dbg!(pos.new_position);
            }
            //dbg!(&*BOARD);

            // Process
            Ok(0u64.into())
        },
    ],
};
