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
            answers = unsigned![123];
        }
        actual_answers = unsigned![123];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use bimap::BiHashMap;
    use id_pool::IdPool;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{alpha1, alphanumeric1},
        combinator::map,
        multi::separated_list1,
        sequence::pair,
    };

    type Id = usize;

    #[derive(Debug)]
    struct Materials {
        id_pool: IdPool,
        map: BiHashMap<Id, String>,
    }
    impl Materials {
        pub fn new() -> Self {
            Self {
                id_pool: IdPool::new(),
                map: BiHashMap::new(),
            }
        }

        pub fn name(&self, id: Id) -> Option<&str> {
            self.map.get_by_left(&id).map(AsRef::as_ref)
        }

        pub fn lookup_or_add(&mut self, name: &str) -> Id {
            match self.map.get_by_right(name) {
                Some(id) => *id,
                None => {
                    let id = self.id_pool.request_id().unwrap();
                    self.map.insert(id, name.into());
                    id
                }
            }
        }
    }

    enum ItemParse<'a> {
        Generator(&'a str),
        Microchip(&'a str),
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
    impl ItemParse<'_> {
        pub fn material_name(&self) -> &str {
            match self {
                ItemParse::Generator(name) => name,
                ItemParse::Microchip(name) => name,
            }
        }

        pub fn into_item(self, materials: &mut Materials) -> Item {
            let id = materials.lookup_or_add(self.material_name());

            match self {
                ItemParse::Generator(_) => Item::Generator(id),
                ItemParse::Microchip(_) => Item::Microchip(id),
            }
        }
    }

    struct FloorParse<'a> {
        name: &'a str,
        items: Vec<ItemParse<'a>>,
    }
    impl<'a> Parsable<'a> for FloorParse<'a> {
        fn parser(input: &'a str) -> NomParseResult<&'a str, Self> {
            map(
                (
                    (tag("The "), alphanumeric1, tag(" floor contains ")),
                    separated_list1(
                        alt((tag(", "), tag(" and "), tag(", and "))),
                        ItemParse::parser,
                    ),
                ),
                |((_, name, _), items)| Self { name, items },
            )
            .parse(input)
        }
    }
    impl FloorParse<'_> {
        pub fn try_into_floor(self, materials: &mut Materials) -> AocResult<(usize, Floor)> {
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
                Floor {
                    items: self
                        .items
                        .into_iter()
                        .map(|ip| ip.into_item(materials))
                        .collect(),
                },
            ))
        }
    }

    struct Floor {
        items: Vec<Item>,
    }

    enum Item {
        Generator(Id),
        Microchip(Id),
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

            // Process
            Ok(0u64.into())
        },
    ],
};
