use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(11120), Unsigned(49232)],
    "#############
#...........#
###B#C#B#D###
#A#D#C#A#
#########",
    vec![12521u64, 44169].answer_vec()
    }
}

mod solution {
    use super::*;
    use crate::aoc::parse::trim;
    use std::{
        cmp::Ordering,
        collections::{BTreeSet, HashMap},
        fmt,
        iter::repeat_with,
        marker::PhantomData,
        ops::Add,
    };

    use enum_map::{Enum, EnumMap};
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

    pub trait Part: Clone + Eq + std::hash::Hash {
        const DEPTH: usize;

        fn board() -> &'static Board<Self>;
        fn add_folded(position_map: &mut PositionMap);
    }
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct PartOne;
    impl Part for PartOne {
        const DEPTH: usize = 2;

        fn board() -> &'static Board<Self> {
            &BOARD_A
        }

        fn add_folded(_position_map: &mut PositionMap) {
            // No folded positions for this part
        }
    }
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct PartTwo;
    impl Part for PartTwo {
        const DEPTH: usize = 4;

        fn board() -> &'static Board<Self> {
            &BOARD_B
        }

        fn add_folded(position_map: &mut PositionMap) {
            // First folded row (DCBA)
            position_map[Amphipod::Desert].insert(Self::board().room_spaces[Amphipod::Amber][1]);
            position_map[Amphipod::Copper].insert(Self::board().room_spaces[Amphipod::Bronze][1]);
            position_map[Amphipod::Bronze].insert(Self::board().room_spaces[Amphipod::Copper][1]);
            position_map[Amphipod::Amber].insert(Self::board().room_spaces[Amphipod::Desert][1]);

            // Second folded row (DBAC)
            position_map[Amphipod::Desert].insert(Self::board().room_spaces[Amphipod::Amber][2]);
            position_map[Amphipod::Bronze].insert(Self::board().room_spaces[Amphipod::Bronze][2]);
            position_map[Amphipod::Amber].insert(Self::board().room_spaces[Amphipod::Copper][2]);
            position_map[Amphipod::Copper].insert(Self::board().room_spaces[Amphipod::Desert][2]);
        }
    }

    #[derive(Debug, Clone, Copy, Enum, EnumIter, PartialEq, Eq)]
    pub enum Amphipod {
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
        fn required_energy(&self) -> u64 {
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
        Room(Amphipod, usize),
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
    pub struct Board<P> {
        graph: Graph,
        hall_spaces: Vec<NodeIndex>,
        room_spaces: EnumMap<Amphipod, Vec<NodeIndex>>,
        _phantom: PhantomData<P>,
    }
    impl<P: Part> Board<P> {
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
                .map(|(ai, amph)| {
                    // Add the room nodes
                    let mut rooms = Vec::with_capacity(P::DEPTH);
                    for ri in 0..P::DEPTH {
                        rooms.push(graph.add_node(SpaceType::Room(amph, ri)));
                        if ri == 0 {
                            graph.add_edge(hall_spaces[ai + 1], rooms[0], 2.into());
                            graph.add_edge(hall_spaces[ai + 2], rooms[0], 2.into());
                        } else {
                            graph.add_edge(rooms[ri - 1], rooms[ri], 1.into());
                        }
                    }

                    // Connect the hall nodes
                    graph.add_edge(hall_spaces[ai + 1], hall_spaces[ai + 2], 2.into());

                    (amph, rooms)
                })
                .collect();

            Self {
                graph,
                hall_spaces,
                room_spaces,
                _phantom: Default::default(),
            }
        }
    }
    impl<P: Part> fmt::Debug for Board<P> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for space in self.hall_spaces.iter() {
                writeln!(f, "Hall space: {:?} {:?}", space, self.graph[*space])?;
            }
            for amph in Amphipod::iter() {
                for (ri, space) in self.room_spaces[amph].iter().enumerate() {
                    writeln!(
                        f,
                        "Room space: {:?} {:?} {:?}",
                        space, ri, self.graph[*space]
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
        static ref BOARD_A: Board<PartOne> = Board::new();
        static ref BOARD_B: Board<PartTwo> = Board::new();
    }
    const BORDER_DISP: &str = "#";
    const EMPTY_DISP: &str = ".";

    #[derive(Debug)]
    struct Move<P: Part + 'static> {
        energy: u64,
        new_position: Position<P>,
    }

    type PositionMap = EnumMap<Amphipod, BTreeSet<NodeIndex>>;

    #[derive(PartialEq, Eq, Clone, Hash)]
    pub struct Position<P> {
        positions: PositionMap,
        _phantom: PhantomData<P>,
    }
    impl<P: Part + 'static> Parseable<'_> for Position<P> {
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
                    let mut position_map: PositionMap = Amphipod::iter()
                        .map(|amph| (amph, BTreeSet::new()))
                        .collect();

                    // Set the first and last rows
                    for (room_amph, (adj_amph, deep_amph)) in
                        Amphipod::iter().zip(rows[0].iter().zip(rows[1].iter()))
                    {
                        position_map[*adj_amph].insert(P::board().room_spaces[room_amph][0]);
                        position_map[*deep_amph]
                            .insert(P::board().room_spaces[room_amph][P::DEPTH - 1]);
                    }

                    // Add folded rows if any
                    P::add_folded(&mut position_map);

                    Position {
                        positions: position_map,
                        _phantom: Default::default(),
                    }
                },
            )(input)
        }
    }
    impl<P: Part + 'static> fmt::Debug for Position<P> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            fmt::Display::fmt(self, f)
        }
    }
    impl<P: Part + 'static> fmt::Display for Position<P> {
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
            fmt_spaces(f, &P::board().hall_spaces[0..2], "")?;
            write!(f, "{EMPTY_DISP}")?;
            for i in 2..5 {
                fmt_spaces(f, &P::board().hall_spaces[i..=i], EMPTY_DISP)?;
            }
            fmt_spaces(f, &P::board().hall_spaces[5..7], "")?;
            writeln!(f, "{BORDER_DISP}")?;

            // Room spaces
            let room_spaces = |space_type| -> Vec<NodeIndex> {
                Amphipod::iter()
                    .map(|amph| P::board().room_spaces[amph][space_type])
                    .collect()
            };
            write!(f, "{}", BORDER_DISP.repeat(3))?;
            fmt_spaces(f, &room_spaces(0), BORDER_DISP)?;
            writeln!(f, "{}", BORDER_DISP.repeat(2))?;
            for depth in 1..P::DEPTH {
                write!(f, "  {BORDER_DISP}")?;
                fmt_spaces(f, &room_spaces(depth), BORDER_DISP)?;
                writeln!(f, "  ")?;
            }
            writeln!(f, "  {}  ", BORDER_DISP.repeat(9))?;

            Ok(())
        }
    }
    impl<P: Part + 'static> Position<P> {
        /// Returns the occupant of a given space, if any.
        fn occupant(&self, space: &NodeIndex) -> Option<Amphipod> {
            for (amph, idxs) in self.positions.iter() {
                if idxs.contains(space) {
                    return Some(amph);
                }
            }
            None
        }

        /// Returns an iterator of all occupied spaces.
        fn occupied_spaces(&self) -> impl Iterator<Item = NodeIndex> + '_ {
            self.positions.values().flatten().copied()
        }

        /// Returns the deepest space in a given room that is unoccupied.
        /// None is returns if every room space is occupied.
        fn deepest_free_space(&self, room: Amphipod) -> Option<NodeIndex> {
            (0..P::DEPTH).rev().find_map(|depth| {
                let node = P::board().room_spaces[room][depth];
                match self.occupant(&node) {
                    Some(_) => None,
                    None => Some(node),
                }
            })
        }

        /// Tests whether or not the position is the solved position with every amphipod in their home room.
        fn solved(&self) -> bool {
            Amphipod::iter().all(|a| {
                P::board().room_spaces[a]
                    .iter()
                    .all(|n| self.positions[a].contains(n))
            })
        }

        /// Moves an amphipod from one space to another.
        fn move_amphipod(&mut self, amphipod: Amphipod, old: &NodeIndex, new: NodeIndex) {
            let nodes = &mut self.positions[amphipod];

            nodes.remove(old);
            nodes.insert(new);
        }

        /// Returns a vector of possible moves for all amphipods.
        fn moves(&self) -> Vec<Move<P>> {
            // NOTE: One principle we follow that is not a rule we never move an amphipod only partially into
            // a room, we always go as deep as possible. Likewise we never move an amphipod to a different space
            // in the same room.
            let mut moves = Vec::new();

            // Go through all amphipods at all (occupied) spaces
            for own_amph in Amphipod::iter() {
                // The nodes of our home room
                let home_spaces = &P::board().room_spaces[own_amph];

                // Whether our home room is filled only with our own kind
                let home_good = home_spaces.iter().all(|n| match self.occupant(n) {
                    Some(a) => a == own_amph,
                    None => true,
                });

                for own_space_node in self.positions[own_amph].iter() {
                    let own_space_type = P::board().graph.node_weight(*own_space_node).unwrap();

                    // If we are already home (and it's filled with like amphipods) then we do not want to move
                    if let SpaceType::Room(own_space_amph, _) = own_space_type && *own_space_amph == own_amph && home_good {
                        continue;
                    }

                    // Remove all occupied graph nodes except this one
                    let mut graph = P::board().clone().graph;
                    self.occupied_spaces().for_each(|n| {
                        if n != *own_space_node {
                            graph.remove_node(n);
                        }
                    });

                    // Also remove all rooms that we don't want to move into
                    for room_amph in Amphipod::iter() {
                        // Do we want to remove or keep this room?
                        if !match own_space_type {
                            // If in the hall, we only want to keep our own room but only if it's filled with our kind
                            SpaceType::Hall => room_amph == own_amph && home_good,
                            // Need to keep only the room we are in or our home room if it's filled with our kind
                            SpaceType::Room(own_space_amph, _) => {
                                room_amph == *own_space_amph || (room_amph == own_amph && home_good)
                            }
                        } {
                            // Remove this entire room
                            P::board().room_spaces[room_amph].iter().for_each(|n| {
                                graph.remove_node(*n);
                            })
                        }
                    }

                    //println!("Amph {} at {}", amphipod, space.index());

                    // Determine shortest paths to all possible destination nodes and filter by those we actually might want to move to
                    let paths = bellman_ford(&graph, *own_space_node).unwrap();
                    for (distance, node) in graph.node_indices().filter_map(|node| {
                        let new_space_type = graph.node_weight(node).unwrap();

                        // Do not want to move to unreachable nodes
                        let d = match paths.distances[node.index()] {
                            Distance::Finite(d) => d,
                            // Node is unreachable
                            Distance::Infinite => return None,
                        };
                        // Do not want to move to our own space
                        if d == 0 {
                            return None;
                        }

                        // Do we want to remove this space?
                        if match new_space_type {
                            // We cannot move to another hall node if we are in the hall but want
                            // to keep hall spaces if we are in a room.
                            SpaceType::Hall => matches!(own_space_type, SpaceType::Hall),
                            // We only want to move into the deepest free space in our home room.
                            SpaceType::Room(room, _) => {
                                !(home_good
                                    && *room == own_amph
                                    && self.deepest_free_space(own_amph).is_some_and(|n| n == node))
                            }
                        } {
                            return None;
                        }

                        Some((d, node))
                    }) {
                        // Copy current position and make the move
                        let mut new_position = self.clone();
                        new_position.move_amphipod(own_amph, own_space_node, node);

                        moves.push(Move {
                            energy: own_amph.required_energy() * u64::from(distance),
                            new_position,
                        })
                    }
                }
            }

            moves
        }

        /// Runs a reursive algorithm to determine the minimum energy needed
        /// to solve from this position.
        pub fn minimal_energy(self) -> AocResult<u64> {
            trait Min<T> {
                fn update_min(&mut self, v: T);
            }
            impl Min<u64> for Option<u64> {
                fn update_min(&mut self, v: u64) {
                    *self = match self {
                        Some(mv) => Some((*mv).min(v)),
                        None => Some(v),
                    };
                }
            }

            // Recursive function
            fn rec<P: Part + 'static>(
                position: Position<P>,
                seen: &mut HashMap<Position<P>, Option<u64>>,
                mut global_min_energy: Option<u64>,
                current_energy: u64,
                _level: u16,
            ) -> Option<u64> {
                // Are we in a solved position?
                if position.solved() {
                    //println!("Level {_level}: Solved!");
                    return Some(0);
                }

                // Have we seen this state before?
                if let Some(e) = seen.get(&position) {
                    return *e;
                }

                // Are we already a larger energy than the global minimium?
                if let Some(e) = global_min_energy && current_energy >= e {
                    return None;
                }

                //println!("Level {_level}:\n{}", position);

                // Recurse for each possible move
                let mut min_energy: Option<u64> = None;
                for mv in position.moves() {
                    if let Some(e) = rec(
                        mv.new_position,
                        seen,
                        global_min_energy,
                        current_energy + mv.energy,
                        _level + 1,
                    ) {
                        min_energy.update_min(e + mv.energy);
                        if let Some(me) = min_energy {
                            global_min_energy.update_min(me);
                        }
                    }
                }

                // Mark this position as seen
                seen.insert(position, min_energy);

                min_energy
            }

            rec(self, &mut HashMap::new(), None, 0, 0).ok_or(AocError::NoSolution)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 23,
    name: "Amphipod",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let pos: solution::Position<PartOne> =
                solution::Position::from_str(input.expect_input()?)?;

            // Process
            Ok(pos.minimal_energy()?.into())
        },
        // Part two
        |input| {
            // Generation
            let pos: solution::Position<PartTwo> =
                solution::Position::from_str(input.expect_input()?)?;

            // Process
            Ok(pos.minimal_energy()?.into())
        },
    ],
};
