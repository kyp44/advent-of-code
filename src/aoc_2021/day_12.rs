use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
        vec![Unsigned(4011), Unsigned(108035)],
    "start-A
start-b
A-c
A-b
b-d
A-end
b-end",
        vec![10u64, 36].answer_vec(),
    "dc-end
HN-start
start-kj
dc-start
dc-HN
LN-dc
HN-end
kj-sa
kj-HN
kj-dc",
    vec![19u64, 103].answer_vec(),
    "fs-end
he-DX
fs-he
start-DX
pj-DX
end-zg
zg-sl
zg-pj
pj-he
RW-he
fs-DX
pj-RW
zg-RW
start-pj
he-WI
zg-he
pj-fs
start-RW",
    vec![226u64, 3509].answer_vec()
        }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use itertools::Itertools;
    use nom::{
        bytes::complete::tag, character::complete::alphanumeric1, combinator::map,
        sequence::separated_pair,
    };
    use std::{borrow::Borrow, collections::HashSet, fmt::Debug, rc::Rc};

    #[derive(PartialEq, Eq, Hash, Copy, Clone)]
    enum CaveType {
        Start,
        End,
        Big,
        Small,
    }
    impl From<&str> for CaveType {
        fn from(s: &str) -> Self {
            match s {
                "start" => Self::Start,
                "end" => Self::End,
                s if s.to_lowercase() == s => Self::Small,
                _ => Self::Big,
            }
        }
    }

    #[derive(Eq)]
    pub struct Cave {
        name: String,
        cave_type: CaveType,
    }
    impl PartialEq for Cave {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
        }
    }
    impl std::hash::Hash for Cave {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.name.hash(state);
        }
    }
    impl Debug for Cave {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.name)
        }
    }
    impl Cave {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                cave_type: name.into(),
            }
        }

        fn has_name(&self, name: &str) -> bool {
            self.name == name
        }

        fn has_type(&self, cave_type: CaveType) -> bool {
            self.cave_type == cave_type
        }
    }

    #[derive(new)]
    struct Passage {
        cave1: Rc<Cave>,
        cave2: Rc<Cave>,
    }
    impl Passage {
        fn connects(&self, cave: &Cave) -> Option<&Cave> {
            if cave == self.cave1.borrow() {
                Some(&self.cave1)
            } else if cave == self.cave2.borrow() {
                Some(&self.cave2)
            } else {
                None
            }
        }
    }

    #[derive(Clone)]
    pub enum SpecialCave {
        None,
        Some { cave: Rc<Cave>, count: u8 },
    }

    pub struct CaveSystem {
        start: Rc<Cave>,
        caves: Vec<Rc<Cave>>,
        passages: Vec<Passage>,
    }
    impl CaveSystem {
        // TODO: move to FromStr impl
        pub fn from_str(s: &str) -> Result<Self, AocError> {
            struct RawPassage<'b> {
                caves: [&'b str; 2],
            }
            impl<'b> Parseable<'b> for RawPassage<'b> {
                fn parser(input: &'b str) -> NomParseResult<&str, Self> {
                    map(
                        separated_pair(alphanumeric1, tag("-"), alphanumeric1),
                        |(a, b)| Self { caves: [a, b] },
                    )(input)
                }
            }

            let raw_passages = RawPassage::gather(s.lines())?;

            // Build all the caves
            let caves: Vec<Rc<Cave>> = raw_passages
                .iter()
                .flat_map(|p| p.caves.iter())
                .unique()
                .map(|name| Rc::new(Cave::new(name)))
                .collect();

            fn find_cave(
                caves: &[Rc<Cave>],
                f: impl Fn(&Cave) -> bool,
            ) -> Result<Rc<Cave>, AocError> {
                caves
                    .iter()
                    .find(|c| f(c))
                    .map(Rc::clone)
                    .ok_or_else(|| AocError::InvalidInput("Could not find cave".into()))
            }
            let passages = raw_passages
                .into_iter()
                .map(|rp| -> Result<_, AocError> {
                    Ok(Passage::new(
                        find_cave(&caves, |c| c.has_name(rp.caves[0]))?,
                        find_cave(&caves, |c| c.has_name(rp.caves[1]))?,
                    ))
                })
                .collect::<Result<_, _>>()?;

            Ok(Self {
                start: find_cave(&caves, |c| c.has_type(CaveType::Start))?.clone(),
                caves,
                passages,
            })
        }

        fn connecting_caves<'b>(&'b self, cave: &'b Cave) -> impl Iterator<Item = &Cave> + 'b {
            self.passages.iter().filter_map(|p| p.connects(cave))
        }

        pub fn paths(&self, special_cave: SpecialCave) -> Result<HashSet<Vec<&Cave>>, AocError> {
            // Paths from the given cave to the end having already visited some
            fn paths_rec<'b>(
                system: &'b CaveSystem,
                cave: &'b Cave,
                visited: &HashSet<&'b Cave>,
                mut special_cave: SpecialCave,
            ) -> Option<HashSet<Vec<&'b Cave>>> {
                let mut paths = HashSet::new();

                if cave.has_type(CaveType::End) {
                    // We've reached the end so this is the only path
                    paths.insert(vec![cave]);
                } else if cave.has_type(CaveType::Small) && visited.contains(cave) {
                    // Cannot revisit this cave so this is invalid
                    return None;
                } else {
                    // We are in a cave that can be revisted (and isn't the end)
                    let mut visited = visited.clone();

                    // If special only mark as visited if allowed visits have been exhausted
                    if let SpecialCave::Some {cave: ref spec_cave, count } = special_cave && cave == spec_cave.borrow() {
                        if count == 1 {
                            visited.insert(cave);
                        } else {
                            special_cave = SpecialCave::Some {cave: spec_cave.clone(), count: count - 1};
                        }
                    } else {
                        visited.insert(cave);
		            }
                    for next_cave in system
                        .connecting_caves(cave)
                        .filter(|c| !c.has_type(CaveType::Start))
                    {
                        if let Some(sub_paths) =
                            paths_rec(system, next_cave, &visited, special_cave.clone())
                        {
                            for mut sub_path in sub_paths.into_iter() {
                                sub_path.insert(0, cave);
                                paths.insert(sub_path);
                            }
                        }
                    }
                }

                Some(paths)
            }

            paths_rec(self, &self.start, &HashSet::new(), special_cave).ok_or(AocError::NoSolution)
        }

        pub fn paths_special(&self) -> Result<HashSet<Vec<&Cave>>, AocError> {
            let mut paths = HashSet::new();
            for cave in self.caves.iter().filter(|c| c.has_type(CaveType::Small)) {
                paths.extend(self.paths(SpecialCave::Some {
                    cave: cave.clone(),
                    count: 2,
                })?)
            }
            Ok(paths)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 12,
    name: "Passage Pathing",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let cave_system = CaveSystem::from_str(input.expect_input()?)?;
            let paths = cave_system.paths(SpecialCave::None)?;

            /*for path in paths.iter() {
                println!("{:?}", path);
            }
            println!();*/

            // Process
            Ok(Answer::Unsigned(paths.len().try_into().unwrap()))
        },
        // Part two
        |input| {
            // Generation
            let cave_system = CaveSystem::from_str(input.expect_input()?)?;
            let paths = cave_system.paths_special()?;

            // Process
            Ok(Answer::Unsigned(paths.len().try_into().unwrap()))
        },
    ],
};
