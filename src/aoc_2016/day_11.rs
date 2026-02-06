use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "The first floor contains a hydrogen-compatible microchip and a lithium-compatible microchip.
The second floor contains a hydrogen generator.
The third floor contains a lithium generator.
The fourth floor contains nothing relevant.";
            answers = unsigned![123];
        }
        actual_answers = unsigned![123];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use bimap::BiHashMap;
    use id_pool::IdPool;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{alpha1, alphanumeric1},
        combinator::map,
        multi::separated_list1,
    };
    use std::{
        cmp::Ordering,
        collections::{HashMap, HashSet},
        hash::Hash,
    };

    type Id = usize;

    #[derive(Debug)]
    struct Materials {
        id_pool: IdPool,
        map: HashMap<String, Id>,
    }
    impl Materials {
        pub fn new() -> Self {
            Self {
                id_pool: IdPool::new(),
                map: HashMap::new(),
            }
        }

        pub fn id_or_add(&mut self, name: &str) -> Id {
            match self.map.get(name) {
                Some(id) => *id,
                None => {
                    let id = self.id_pool.request_id().unwrap();
                    self.map.insert(name.into(), id);
                    id
                }
            }
        }
    }

    #[derive(Debug)]
    enum ItemParse<'a> {
        Generator(&'a str),
        Microchip(&'a str),
    }
    impl ItemParse<'_> {
        pub fn material_name(&self) -> &str {
            match self {
                ItemParse::Generator(name) => name,
                ItemParse::Microchip(name) => name,
            }
        }

        pub fn into_item(self, materials: &mut Materials) -> Item {
            let id = materials.id_or_add(self.material_name());

            match self {
                ItemParse::Generator(_) => Item::Generator(id),
                ItemParse::Microchip(_) => Item::Microchip(id),
            }
        }
    }
    impl<'a> Parsable<'a> for ItemParse<'a> {
        fn parser(input: &'a str) -> NomParseResult<&'a str, Self> {
            alt((
                map(
                    (tag("a "), alpha1, tag(" generator")),
                    |(_, material, _)| Self::Generator(material),
                ),
                map(
                    (tag("a "), alpha1, tag("-compatible microchip")),
                    |(_, material, _)| Self::Microchip(material),
                ),
            ))
            .parse(input)
        }
    }

    #[derive(PartialEq, Eq, Hash)]
    enum Item {
        Generator(Id),
        Microchip(Id),
    }
    impl Item {
        pub fn variant_id(&self) -> usize {
            match self {
                Item::Generator(_) => 0,
                Item::Microchip(_) => 1,
            }
        }

        pub fn id(&self) -> Id {
            match self {
                Item::Generator(id) => *id,
                Item::Microchip(id) => *id,
            }
        }

        pub fn corresponding(&self) -> Self {
            match self {
                Item::Generator(id) => Self::Microchip(*id),
                Item::Microchip(id) => Self::Generator(*id),
            }
        }
    }
    impl std::fmt::Debug for Item {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}{}",
                match self {
                    Item::Generator(_) => "G",
                    Item::Microchip(_) => "M",
                },
                self.id()
            )
        }
    }
    impl PartialOrd for Item {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Item {
        fn cmp(&self, other: &Self) -> Ordering {
            (self.variant_id(), self.id()).cmp(&(other.variant_id(), other.id()))
        }
    }

    #[derive(Debug)]
    struct FloorParse<'a> {
        name: &'a str,
        items: Vec<ItemParse<'a>>,
    }
    impl FloorParse<'_> {
        pub fn floor_items(self, materials: &mut Materials) -> AocResult<(usize, Vec<Item>)> {
            let floor_num = match self.name {
                "first" => Ok(0),
                "second" => Ok(1),
                "third" => Ok(2),
                "fourth" => Ok(3),
                _ => Err(AocError::InvalidInput(
                    format!("Floor '{}' is not a valid floor number", self.name).into(),
                )),
            }?;

            Ok((
                floor_num,
                self.items
                    .into_iter()
                    .map(|ip| ip.into_item(materials))
                    .collect(),
            ))
        }
    }
    impl<'a> Parsable<'a> for FloorParse<'a> {
        fn parser(input: &'a str) -> NomParseResult<&'a str, Self> {
            map(
                trim(
                    true,
                    (
                        (tag("The "), alphanumeric1, tag(" floor contains ")),
                        alt((
                            separated_list1(
                                alt((tag(", "), tag(" and "), tag(", and "))),
                                ItemParse::parser,
                            ),
                            map(tag("nothing relevant"), |_| Vec::new()),
                        )),
                        tag("."),
                    ),
                ),
                |((_, name, _), items, _)| Self { name, items },
            )
            .parse(input)
        }
    }

    pub struct RtgState {
        // TODO: This needs to be a vec of items!
        positions: BiHashMap<Itemssss, u8>,
    }
    impl RtgState {
        fn floor_items(&self, floor: u8) -> HashSet<&Item> {
            assert!(floor < 4);
            HashSet::from_iter(
                self.positions
                    .iter()
                    .filter_map(|(item, n)| (*n == floor).some(item)),
            )
        }

        fn possible_moves(&self) -> Vec<Self> {
            todo!()
        }
    }
    impl FromStr for RtgState {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut materials = Materials::new();

            // Parse the floors
            let floors = FloorParse::gather(s.lines())?
                .into_iter()
                .map(|f| f.floor_items(&mut materials))
                .collect::<AocResult<Vec<_>>>()?;
            println!("TODO all floors: {floors:?}");

            // Verify the that exactly all floors are present
            let mut floor_numbers: Vec<_> = floors.iter().map(|f| f.0).collect();
            floor_numbers.sort();
            (floor_numbers == [0, 1, 2, 3]).ok_or(AocError::InvalidInput(
                "The input does not contain exactly the four floors".into(),
            ))?;

            // Build the positions map
            let mut positions = BiHashMap::new();
            for (floor_num, items) in floors.into_iter() {
                println!("TODO wtf: {floor_num} {items:?}");
                for item in items {
                    (!positions.contains_left(&item)).ok_or_else(|| {
                        AocError::InvalidInput(format!("There are two of the {item:?}!").into())
                    })?;
                    println!("TODO one item: {floor_num} {item:?}");

                    positions.insert(item, u8::try_from(floor_num).unwrap());
                    println!("TODO poses: {positions:?}");
                }
            }

            // Verify that every item its corresponding item
            println!("TODO positions {positions:?}");
            for item in positions.left_values() {
                let item_cor = item.corresponding();

                (!positions.contains_left(&item_cor)).ok_or_else(|| {
                    AocError::InvalidInput(format!("The {item:?} was not found").into())
                })?;
            }

            Ok(Self { positions })
        }
    }
    impl std::fmt::Debug for RtgState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for floor in 4u8..0 {
                writeln!(f, "{floor}")?;
            }
            Ok(())
        }
    }

    pub fn test(input: &str) {
        let mut materials = Materials::new();
        let fps = FloorParse::gather(input.lines()).unwrap();
        for fp in fps {
            println!("TODO: {:?}", fp);
            println!("TODO: {:?}", fp.floor_items(&mut materials));
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 11,
    name: "Radioisotope Thermoelectric Generators",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            test(input.expect_text()?);
            let initial_state = RtgState::from_str(input.expect_text()?)?;
            println!("TODO:\n{initial_state:?}");

            // Process
            Ok(0u64.into())
        },
    ],
};
