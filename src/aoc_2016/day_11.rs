use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "The first floor contains a hydrogen-compatible microchip and a lithium-compatible microchip.
The second floor contains a hydrogen generator.
The third floor contains a lithium generator.
The fourth floor contains nothing relevant.";
            answers = unsigned![11];
        }
        example {
            input = "The first floor contains a hydrogen generator and a lithium generator.
The second floor contains a hydrogen-compatible microchip.
The third floor contains a lithium-compatible microchip.
The fourth floor contains nothing relevant.";
            answers = unsigned![9, 33];
        }
        actual_answers = unsigned![31, 55];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use derive_more::Display;
    use derive_new::new;
    use itertools::{Itertools, iproduct};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{alpha1, alphanumeric1},
        combinator::map,
        multi::separated_list1,
    };
    use petgraph::Graph;
    use sorted_vec::SortedVec;
    use std::{
        cmp::Ordering,
        collections::{HashMap, HashSet},
        hash::Hash,
    };

    /// The type for an [`Item`] ID number.
    type Id = u8;
    /// The type for a floor number.
    type FloorNum = u8;
    /// The number of floors in the problem.
    const NUM_FLOORS: FloorNum = 4;

    /// A single item that can be parsed from text.
    #[derive(Debug, PartialEq, Eq, Hash)]
    enum ItemParse<'a> {
        /// A Radioisotope Thermoelectric Generator (RTG) with its material
        /// name.
        Generator(&'a str),
        /// A microchip with its material name.
        Microchip(&'a str),
    }
    impl ItemParse<'_> {
        /// Returns the material name for the item.
        pub fn material_name(&self) -> &str {
            match self {
                ItemParse::Generator(name) => name,
                ItemParse::Microchip(name) => name,
            }
        }
    }
    impl Parsable for ItemParse<'_> {
        type Parsed<'a> = ItemParse<'a>;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            alt((
                map(
                    (tag("a "), alpha1, tag(" generator")),
                    |(_, material, _)| ItemParse::Generator(material),
                ),
                map(
                    (tag("a "), alpha1, tag("-compatible microchip")),
                    |(_, material, _)| ItemParse::Microchip(material),
                ),
            ))
            .parse(input)
        }
    }

    /// An entire floor that can be parsed from text.
    #[derive(Debug)]
    struct FloorParse<'a> {
        /// The floor number as a name.
        name: &'a str,
        /// The parsed items on the floor.
        items: HashSet<ItemParse<'a>>,
    }
    impl FloorParse<'_> {
        /// Returns the floor number as a number, or an `Err` if the floor
        /// number name does not correspond to a valid number.
        pub fn floor_num(&self) -> AocResult<FloorNum> {
            match self.name {
                "first" => Ok(0),
                "second" => Ok(1),
                "third" => Ok(2),
                "fourth" => Ok(3),
                _ => Err(AocError::InvalidInput(
                    format!("Floor '{}' is not a valid floor number", self.name).into(),
                )),
            }
        }
    }
    impl Parsable for FloorParse<'_> {
        type Parsed<'a> = FloorParse<'a>;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(
                trim(
                    true,
                    (
                        (tag("The "), alphanumeric1, tag(" floor contains ")),
                        alt((
                            separated_list1(
                                alt((tag(", and "), tag(", "), tag(" and "))),
                                ItemParse::parser,
                            ),
                            map(tag("nothing relevant"), |_| Vec::new()),
                        )),
                        tag("."),
                    ),
                ),
                |((_, name, _), items, _)| FloorParse {
                    name,
                    items: items.into_iter().collect(),
                },
            )
            .parse(input)
        }
    }

    /// An item that cannot be parsed from text.
    #[derive(Clone, Copy, Display, PartialEq, Eq, Hash)]
    enum Item {
        /// A Radioisotope Thermoelectric Generator (RTG) with material ID
        /// number.
        #[display("G{_0}")]
        Generator(Id),
        /// A microchip with its material ID number.
        #[display("M{_0}")]
        Microchip(Id),
    }
    impl Item {
        /// Returns a number corresponding with the item variant, used for
        /// sorting.
        fn variant_id(&self) -> usize {
            match self {
                Self::Generator(_) => 0,
                Self::Microchip(_) => 1,
            }
        }

        /// Returns the material ID number for the item.
        pub fn id(&self) -> Id {
            match self {
                Self::Generator(id) => *id,
                Self::Microchip(id) => *id,
            }
        }

        /// Returns the corresponding item with the same material ID number.
        pub fn corresponding(&self) -> Self {
            match self {
                Self::Generator(id) => Self::Microchip(*id),
                Self::Microchip(id) => Self::Generator(*id),
            }
        }
    }
    impl std::fmt::Debug for Item {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Display::fmt(self, f)
        }
    }
    impl PartialOrd for Item {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Item {
        fn cmp(&self, other: &Self) -> Ordering {
            (self.id(), self.variant_id()).cmp(&(other.id(), other.variant_id()))
        }
    }

    /// A corresponding pair of items and which floor they sit on.
    #[derive(new, Clone, Debug)]
    struct ItemPair {
        /// The material ID number for the pair.
        ///
        /// Note that this is not used for the purposes of equality and order.
        id: Id,
        /// The floor that the generator is on.
        generator_floor: FloorNum,
        /// The floor that the microchip is on.
        microchip_floor: FloorNum,
    }
    impl ItemPair {
        /// Returns the [`Item`]s for this pair that are on the `floor`.
        ///
        /// The returned [`Vec`] will be be empty if neither item is on the
        /// `floor`.
        pub fn items(&self, floor: FloorNum) -> Vec<Item> {
            let mut items = Vec::new();

            if self.generator_floor == floor {
                items.push(Item::Generator(self.id));
            }
            if self.microchip_floor == floor {
                items.push(Item::Microchip(self.id));
            }

            items
        }
    }
    impl PartialEq for ItemPair {
        fn eq(&self, other: &Self) -> bool {
            self.generator_floor == other.generator_floor
                && self.microchip_floor == other.microchip_floor
        }
    }
    impl Eq for ItemPair {}
    impl Hash for ItemPair {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.generator_floor.hash(state);
            self.microchip_floor.hash(state);
        }
    }
    impl PartialOrd for ItemPair {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for ItemPair {
        fn cmp(&self, other: &Self) -> Ordering {
            (self.generator_floor, self.microchip_floor)
                .cmp(&(other.generator_floor, other.microchip_floor))
        }
    }

    /// The collection of [`ItemPair`]s.
    ///
    /// This is always sorted so that pairs with different material ID numbers
    /// can effectively be interchanged freely, see
    /// [`minimal_solution`](RtgState::minimal_solution).
    type ItemPairs = SortedVec<ItemPair>;

    /// A state of the Radioisotope Testing Facility.
    ///
    /// This can be parsed from text input.
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct RtgState {
        /// The floor on which is the elevator.
        elevator_floor: FloorNum,
        /// The positions of all the items.
        item_pairs: ItemPairs,
    }
    impl FromStr for RtgState {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // Parse the floors and get all materials
            let floors = FloorParse::gather(s.lines())?;
            let materials = floors
                .iter()
                .flat_map(|f| f.items.iter().map(|i| i.material_name()))
                .collect::<HashSet<_>>();

            // Verify the that exactly all floors are present
            let mut floor_numbers = floors
                .iter()
                .map(|f| f.floor_num())
                .collect::<AocResult<Vec<_>>>()?;
            floor_numbers.sort();
            (floor_numbers == [0, 1, 2, 3]).ok_or(AocError::InvalidInput(
                "The input does not contain exactly the four floors".into(),
            ))?;

            // Finds an item and returns the floor that it is on and verifies that the item
            // appears exactly once.
            let find_item = |item: &ItemParse| -> AocResult<u8> {
                let found_items = floors
                    .iter()
                    .filter_map(|f| f.items.contains(item).then_some(f.floor_num().unwrap()))
                    .collect::<Vec<_>>();

                (found_items.len() == 1).ok_or_else(|| {
                    AocError::InvalidInput(
                        format!("Exactly one of the '{item:?}' was not found").into(),
                    )
                })?;

                Ok(found_items[0])
            };

            // Build the item pair table
            let item_pairs = materials
                .into_iter()
                .enumerate()
                .map(|(id, material)| {
                    let (gen_floor, chip_floor) = (
                        find_item(&ItemParse::Generator(material))?,
                        find_item(&ItemParse::Microchip(material))?,
                    );

                    Ok(ItemPair::new(
                        (id + 1).try_into().unwrap(),
                        gen_floor,
                        chip_floor,
                    ))
                })
                .collect::<AocResult<_>>()?;

            Ok(Self {
                elevator_floor: 0,
                item_pairs,
            })
        }
    }
    impl std::fmt::Debug for RtgState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let all_items = self.all_items().collect::<SortedVec<_>>();
            writeln!(f, "State hash: {}", self.state_hash())?;
            for floor in (0..NUM_FLOORS).rev() {
                let items = self.floor_items(floor);
                write!(
                    f,
                    "F{} {}  ",
                    floor + 1,
                    if floor == self.elevator_floor {
                        "E"
                    } else {
                        "."
                    }
                )?;
                for item in all_items.iter() {
                    if items.contains(item) {
                        write!(f, "{item} ")?;
                    } else {
                        write!(f, ".  ")?;
                    }
                }
                writeln!(f)?;
            }
            Ok(())
        }
    }
    impl RtgState {
        /// Returns all of the [`Item`]s in the facility in no particular order.
        fn all_items(&self) -> impl Iterator<Item = Item> {
            self.item_pairs
                .iter()
                .flat_map(|ip| [Item::Generator(ip.id), Item::Microchip(ip.id)])
        }

        /// Returns only the [`Item`]s on the `floor` as a set.
        fn floor_items(&self, floor: FloorNum) -> HashSet<Item> {
            debug_assert!(floor < NUM_FLOORS);

            self.item_pairs
                .iter()
                .flat_map(|ip| ip.items(floor))
                .collect()
        }

        /// Returns all of the items in the facility organized by floor, in
        /// order of increasing floor numbers.
        fn all_floor_items(&self) -> Vec<HashSet<Item>> {
            (0..NUM_FLOORS).map(|f| self.floor_items(f)).collect()
        }

        /// Returns whether this state is a valid one.
        ///
        /// This means that the elevator must be on a floor with one or more
        /// items, and that no microchips are on the same floor with any
        /// mismatched generators unless it is plugged into its
        /// corresponding generator (that is, they are on the same floor).
        fn is_valid(&self) -> bool {
            let all_floor_items = self.all_floor_items();

            // Verify that no microchips are with other generators without being connected
            // to their own generator
            for floor_items in all_floor_items.iter() {
                let (chips, gens): (HashSet<_>, HashSet<_>) =
                    floor_items.iter().copied().partition(|item| match item {
                        Item::Generator(_) => false,
                        Item::Microchip(_) => true,
                    });
                for chip in chips {
                    if gens.contains(&chip.corresponding()) {
                        continue;
                    } else if !gens.is_empty() {
                        return false;
                    }
                }
            }

            // Verify that the elevator is not on a floor with no items, which is impossible
            !self.floor_items(self.elevator_floor).is_empty()
        }

        /// Returns the adjacent floors.
        ///
        /// This includes the optimization that the floor below is not included
        /// if all the floors below the current floor are are empty (that is,
        /// have no items). See
        /// [`minimal_solution`](RtgState::minimal_solution).
        fn possible_movement_floors(&self) -> Vec<u8> {
            let floor = self.elevator_floor;
            let mut floors = Vec::new();

            if floor > 0 {
                // Do not move anything down if the floors below are already empty
                if (0..floor).any(|f| !self.floor_items(f).is_empty()) {
                    floors.push(floor - 1);
                }
            }
            if floor < NUM_FLOORS - 1 {
                floors.push(floor + 1);
            }

            floors
        }

        /// Returns a new state with some `items` on the current floor moved to
        /// a `new_floor`.
        ///
        /// Note that the items are not checked to ensure that they are even on
        /// the current floor.
        fn move_elevator_and_items<'a, I: IntoIterator<Item = &'a Item>>(
            &self,
            items: I,
            new_floor: u8,
        ) -> Self {
            debug_assert!(new_floor < NUM_FLOORS);

            // Move each item one at a time
            let mut new = self.clone();
            new.item_pairs.mutate_vec(|vec| {
                for item in items.into_iter() {
                    let item_pair = vec.iter_mut().find(|ip| ip.id == item.id()).unwrap();

                    match item {
                        Item::Generator(_) => item_pair.generator_floor = new_floor,
                        Item::Microchip(_) => item_pair.microchip_floor = new_floor,
                    }
                }

                vec.sort_unstable();
            });

            // Move the elevator as well
            new.elevator_floor = new_floor;
            new
        }

        /// Returns possible _valid_ states that can be reached by moving items
        /// on the elevator.
        ///
        /// This does not include inadvisable moves according to the
        /// optimizations. See [`minimal_solution`](RtgState::minimal_solution).
        fn possible_moves(&self) -> Vec<Self> {
            let mut moves = Vec::new();
            let item_combos = self
                .floor_items(self.elevator_floor)
                .into_iter()
                .powerset()
                .filter(|items: &Vec<Item>| !items.is_empty() && items.len() <= 2)
                .collect::<Vec<_>>();

            for new_floor in self.possible_movement_floors() {
                moves.extend(item_combos.iter().filter_map(|items| {
                    let new_state = self.move_elevator_and_items(items, new_floor);

                    new_state.is_valid().some(new_state)
                }));
            }

            moves
        }

        /// Returns the goal state for the facility, that is, with all the items
        /// on the top floor.
        fn goal_state(&self) -> Self {
            const TOP_FLOOR: u8 = 3;
            Self {
                elevator_floor: TOP_FLOOR,
                item_pairs: self
                    .item_pairs
                    .iter()
                    .map(|ip| ItemPair::new(ip.id, TOP_FLOOR, TOP_FLOOR))
                    .collect(),
            }
        }

        /// Returns an [`Iterator`] over all valid states.
        ///
        /// This accounts for the optimization that material ID numbers can be
        /// interchanged freely, see
        /// [`minimal_solution`](RtgState::minimal_solution).
        fn all_valid_states(&self) -> impl Iterator<Item = Self> {
            let possible_item_pairs =
                iproduct!(0..NUM_FLOORS, 0..NUM_FLOORS).map(|(gf, mf)| ItemPair::new(0, gf, mf));

            iproduct!(
                0..NUM_FLOORS,
                possible_item_pairs.combinations_with_replacement(self.item_pairs.len())
            )
            .filter_map(|(floor, pairs)| {
                // Need to set Ids
                let mut item_pairs: ItemPairs = pairs.into();
                item_pairs.mutate_vec(|vec| {
                    vec.iter_mut()
                        .enumerate()
                        .for_each(|(id, ip)| ip.id = id.try_into().unwrap());

                    vec.sort_unstable();
                });

                let state = Self {
                    elevator_floor: floor,
                    item_pairs,
                };

                state.is_valid().some(state)
            })
        }

        /// Calculates the hash for the state, which is useful when debugging.
        fn state_hash(&self) -> u64 {
            use std::hash::Hasher;
            let mut hasher = std::hash::DefaultHasher::new();
            self.hash(&mut hasher);
            hasher.finish()
        }

        /// Returns the minimum number of moves required to get from this state
        /// to the goal state with all items on the top floor.
        ///
        /// This is done by building a graph of all states and moves, then using
        /// [Dijkstra's algorithm](https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm)
        /// to find the shortest path through the states.
        ///
        /// There are three main optimizations that speed this up by reducing
        /// the number of needed states/nodes:
        /// 1. Never move things down floors when all the floors below are
        ///    empty.
        /// 2. If you can move two items up there is no reason to just move one,
        ///    but this only applies to up, not down.
        /// 3. All item pairs are interchangeable, which is to say that item
        ///    materials and their ID numbers can be swapped around with no
        ///    meaningful effect. It is still important which microchip goes
        ///    with which generator.
        ///
        /// Optimizations were researched and the above were found on the
        /// [`/r/adventofcode` subreddit](https://www.reddit.com/r/adventofcode/comments/5hoia9/comment/db1v1ws/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button).
        ///
        /// Regarding these optimizations, respectively:
        /// 1. This was implemented and found to only have a minimal impact.
        /// 2. This was was implemented but found to slow things down more than
        ///    speed them up due to the extra code and allocations needed, so
        ///    was removed.
        /// 3. This was implemented and _massively_ reduces the number of states
        ///    and therefore the execution time.
        pub fn minimal_solution(&self) -> AocResult<u64> {
            let mut graph = Graph::new();
            let all_states = self.all_valid_states().collect::<Vec<_>>();
            let mut state_index_map = HashMap::new();

            // Add all the nodes/states
            for state in all_states.iter() {
                state_index_map.insert(state, graph.add_node(state));
            }

            // Build the edges
            for index in graph.node_indices() {
                for next_state in graph.node_weight(index).unwrap().possible_moves() {
                    graph.add_edge(index, *state_index_map.get(&next_state).unwrap(), ());
                }
            }

            // Execute Dijkstra's algorithm to calculate the minimum distance'
            let goal_node = *state_index_map.get(&self.goal_state()).unwrap();
            let shortest_paths = petgraph::algo::dijkstra(
                &graph,
                *state_index_map.get(self).unwrap(),
                Some(goal_node),
                |_| 1u32,
            );

            Ok((*shortest_paths.get(&goal_node).unwrap()).into())
        }

        /// Returns a new state with the unlisted items from part two added.
        pub fn add_unlisted_items(&self) -> Self {
            let mut new = self.clone();

            // Find highest id
            let max_id = new.item_pairs.iter().map(|ip| ip.id).max().unwrap();
            for a in 1..=2 {
                let id = max_id + a;
                new.item_pairs.insert(ItemPair::new(id, 0, 0));
            }

            new
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 11,
    name: "Radioisotope Thermoelectric Generators",
    preprocessor: Some(|input| Ok(Box::new(RtgState::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<RtgState>()?.minimal_solution()?.into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<RtgState>()?
                .add_unlisted_items()
                .minimal_solution()?
                .into())
        },
    ],
};
