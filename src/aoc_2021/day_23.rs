use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "#############
#...........#
###B#C#B#D###
#A#D#C#A#
#########";
            answers = vec![12521u64, 44169].answer_vec();
        }
        actual_answers = vec![Unsigned(11120), Unsigned(49232)];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{
        parse::trim,
        tree_search::{BestMetricTreeNode, Metric, MetricChild},
    };
    use derive_more::{Add, Deref, From};
    use enum_map::{Enum, EnumMap};
    use infinitable::Infinitable;
    use lazy_static::lazy_static;
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
    use std::{collections::BTreeSet, fmt, iter::repeat_with, marker::PhantomData};
    use strum::IntoEnumIterator;
    use strum_macros::EnumIter;

    /// Behavior specific to one particular part of the problem.
    pub trait Part: Clone + Eq + std::hash::Hash {
        /// The depth of the amphipod rooms, which is the number of spaces in each room.
        const DEPTH: usize;

        /// Returns the board for this part.
        fn board() -> &'static Board<Self>;

        /// Adds in additional amphipod positions from the folded part of the diagram.
        fn add_folded(position_map: &mut PositionMap);
    }

    /// Behavior for part one.
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct PartOne;
    impl Part for PartOne {
        const DEPTH: usize = 2;

        fn board() -> &'static Board<Self> {
            &BOARD_ONE
        }

        fn add_folded(_position_map: &mut PositionMap) {
            // No folded positions for this part
        }
    }

    /// Behavior for part two.
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct PartTwo;
    impl Part for PartTwo {
        const DEPTH: usize = 4;

        fn board() -> &'static Board<Self> {
            &BOARD_TWO
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

    /// An amphipod, which can be created from a [`char`].
    #[derive(Debug, Clone, Copy, Enum, EnumIter, PartialEq, Eq)]
    pub enum Amphipod {
        /// An Amber amphipod (`A`).
        Amber,
        /// An Bronze amphipod (`B`).
        Bronze,
        /// An Copper amphipod (`C`).
        Copper,
        /// An Desert amphipod (`D`).
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
        /// Returns the energy required for this amphipod to move one space.
        fn required_energy(&self) -> u64 {
            match self {
                Amphipod::Amber => 1,
                Amphipod::Bronze => 10,
                Amphipod::Copper => 100,
                Amphipod::Desert => 1000,
            }
        }
    }

    /// The type of space on a board.
    #[derive(Debug, Clone)]
    enum SpaceType {
        /// A hall space.
        Hall,
        /// A room space with the home amphipod type, and the position of the space,
        /// with 0 being adjacent to the hall and highest number being the deepest.
        Room(Amphipod, usize),
    }

    /// The distance between two graph nodes.
    #[derive(Deref, From, Add, Clone, Copy, Debug, PartialEq, PartialOrd)]
    struct Distance(Infinitable<u8>);
    impl Default for Distance {
        fn default() -> Self {
            Self::zero()
        }
    }
    impl FloatMeasure for Distance {
        fn zero() -> Self {
            Self(0.into())
        }

        fn infinite() -> Self {
            Self(Infinitable::Infinity)
        }
    }

    /// The type of the graph used to model the board.
    type Graph = StableUnGraph<SpaceType, Distance>;

    /// The board on which the amphipods move.
    #[derive(Clone)]
    pub struct Board<P> {
        /// The graph model of the board.
        graph: Graph,
        /// The graph nodes for the hall spaces.
        hall_spaces: Vec<NodeIndex>,
        /// Map from the amphipod type to their home room space graph nodes.
        room_spaces: EnumMap<Amphipod, Vec<NodeIndex>>,
        /// Phantom  data for the part of the problem.
        _phantom: PhantomData<P>,
    }
    impl<P: Part> Board<P> {
        /// Creates a new board for the part of the problem.
        fn new() -> Self {
            let mut graph = Graph::with_capacity(15, 18);

            // Some oft-used [`Infinitable`] values
            let inf_one = Infinitable::from(1).into();
            let inf_two = Infinitable::from(2).into();

            // All the hall spaces
            let hall_spaces: Vec<_> = repeat_with(|| graph.add_node(SpaceType::Hall))
                .take(7)
                .collect();

            // Connect the end hall spaces
            graph.add_edge(hall_spaces[0], hall_spaces[1], inf_one);
            graph.add_edge(hall_spaces[5], hall_spaces[6], inf_one);

            // Build and connect the side rooms
            let room_spaces = Amphipod::iter()
                .enumerate()
                .map(|(ai, amph)| {
                    // Add the room nodes
                    let mut rooms = Vec::with_capacity(P::DEPTH);
                    for ri in 0..P::DEPTH {
                        rooms.push(graph.add_node(SpaceType::Room(amph, ri)));
                        if ri == 0 {
                            graph.add_edge(hall_spaces[ai + 1], rooms[0], inf_two);
                            graph.add_edge(hall_spaces[ai + 2], rooms[0], inf_two);
                        } else {
                            graph.add_edge(rooms[ri - 1], rooms[ri], inf_one);
                        }
                    }

                    // Connect the hall nodes
                    graph.add_edge(hall_spaces[ai + 1], hall_spaces[ai + 2], inf_two);

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
        /// Board for part one.
        static ref BOARD_ONE: Board<PartOne> = Board::new();
        /// Board for part two.
        static ref BOARD_TWO: Board<PartTwo> = Board::new();
    }

    /// Character to use for displaying the border of the board.
    const BORDER_DISP: &str = "#";
    /// Character to use for displaying an empty space on the board.
    const EMPTY_DISP: &str = ".";

    /// Map of amphipod types to the nodes occupied by those amphipods.
    type PositionMap = EnumMap<Amphipod, BTreeSet<NodeIndex>>;

    /// The positions of all amphipods, which can be parsed from text input.
    #[derive(PartialEq, Eq, Clone, Hash)]
    pub struct Position<P> {
        /// The position map of the amphipods.
        positions: PositionMap,
        /// Phantom data for the part of the problem.
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

        /// Returns an [`Iterator`] over all occupied spaces.
        fn occupied_spaces(&self) -> impl Iterator<Item = NodeIndex> + '_ {
            self.positions.values().flatten().copied()
        }

        /// Returns the deepest space in a given home room that is unoccupied.
        ///
        /// `None` is returned if every home room space is occupied.
        fn deepest_free_space(&self, room: Amphipod) -> Option<NodeIndex> {
            (0..P::DEPTH).rev().find_map(|depth| {
                let node = P::board().room_spaces[room][depth];
                match self.occupant(&node) {
                    Some(_) => None,
                    None => Some(node),
                }
            })
        }

        /// Moves an amphipod from one space to another.
        fn move_amphipod(&mut self, amphipod: Amphipod, old: &NodeIndex, new: NodeIndex) {
            let nodes = &mut self.positions[amphipod];

            nodes.remove(old);
            nodes.insert(new);
        }

        /// Runs the tree search and returns the minimum energy needed
        /// to solve from this position, that is to return all amphipods to their
        /// home rooms.
        pub fn minimal_energy(self) -> AocResult<u64> {
            match self.best_metric().0 {
                Infinitable::Finite(e) => Ok(e),
                _ => Err(AocError::NoSolution),
            }
        }
    }

    #[derive(Clone, Copy, Add, Debug)]
    pub struct Cost(Infinitable<u64>);
    impl Metric for Cost {
        const INITIAL_BEST: Self = Cost(Infinitable::Infinity);
        const INITIAL_COST: Self = Cost(Infinitable::Finite(0));

        fn is_better(&self, other: &Self) -> bool {
            self.0 < other.0
        }
    }
    impl From<u64> for Cost {
        fn from(value: u64) -> Self {
            Self(value.into())
        }
    }
    impl<P: Part + 'static> BestMetricTreeNode for Position<P> {
        type Metric = Cost;

        fn end_state(&self) -> bool {
            Amphipod::iter().all(|a| {
                P::board().room_spaces[a]
                    .iter()
                    .all(|n| self.positions[a].contains(n))
            })
        }

        fn children(&self, _cumulative_cost: &Self::Metric) -> Vec<MetricChild<Self>> {
            // NOTE: One principle we follow that is not a rule, we never move an amphipod only partially into
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
                        let d = match paths.distances[node.index()].finite() {
                            Some(d) => d,
                            None => return None,
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

                        moves.push(MetricChild::new(
                            new_position,
                            (own_amph.required_energy() * u64::from(distance)).into(),
                        ))
                    }
                }
            }

            moves
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 23,
    name: "Amphipod",
    // NOTE: Cannot pre-parse because each Position has a generic part.
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let pos: Position<PartOne> = Position::from_str(input.expect_input()?)?;

            // Process
            Ok(pos.minimal_energy()?.into())
        },
        // Part two
        |input| {
            // Generation
            let pos: Position<PartTwo> = Position::from_str(input.expect_input()?)?;

            // Process
            Ok(pos.minimal_energy()?.into())
        },
    ],
};
