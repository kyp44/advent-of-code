use std::{borrow::Borrow, collections::HashSet, fmt::Debug, rc::Rc};

use itertools::Itertools;

use nom::{
    bytes::complete::tag, character::complete::alphanumeric1, combinator::map,
    sequence::separated_pair,
};

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
struct Cave<'a> {
    name: &'a str,
    cave_type: CaveType,
}
impl PartialEq for Cave<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl std::hash::Hash for Cave<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
impl Debug for Cave<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl<'a> Cave<'a> {
    fn new(name: &'a str) -> Self {
        Self {
            name,
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

struct Passage<'a> {
    caves: (Rc<Cave<'a>>, Rc<Cave<'a>>),
}
impl<'a> Passage<'a> {
    fn new(a: &Rc<Cave<'a>>, b: &Rc<Cave<'a>>) -> Self {
        Self {
            caves: (a.clone(), b.clone()),
        }
    }

    fn connects(&self, cave: &Cave<'a>) -> Option<&Cave<'a>> {
        if cave == self.caves.0.borrow() {
            Some(&self.caves.1)
        } else if cave == self.caves.1.borrow() {
            Some(&self.caves.0)
        } else {
            None
        }
    }
}

#[derive(Clone)]
enum SpecialCave<'a, 'c> {
    None,
    Some { cave: &'a Cave<'c>, count: u8 },
}

struct CaveSystem<'a> {
    start: Rc<Cave<'a>>,
    caves: Vec<Rc<Cave<'a>>>,
    passages: Vec<Passage<'a>>,
}
impl<'a> CaveSystem<'a> {
    fn from_str(s: &'a str) -> Result<Self, AocError> {
        struct RawPassage<'b> {
            caves: [&'b str; 2],
        }
        impl<'b> Parseable<'b> for RawPassage<'b> {
            fn parser(input: &'b str) -> NomParseResult<Self> {
                map(
                    separated_pair(alphanumeric1, tag("-"), alphanumeric1),
                    |(a, b)| Self { caves: [a, b] },
                )(input)
            }
        }

        let raw_passages = RawPassage::gather(s.lines())?;

        // Build all the caves
        let caves: Vec<Rc<Cave<'_>>> = raw_passages
            .iter()
            .flat_map(|p| p.caves.iter())
            .unique()
            .map(|name| Rc::new(Cave::new(name)))
            .collect();

        fn find_cave<'b, 'c>(
            caves: &'b [Rc<Cave<'c>>],
            f: impl Fn(&Cave) -> bool,
        ) -> Result<&'b Rc<Cave<'c>>, AocError> {
            caves
                .iter()
                .find(|c| f(c))
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

    fn connecting_caves<'b>(&'b self, cave: &'b Cave<'a>) -> impl Iterator<Item = &Cave<'a>> + 'b {
        self.passages.iter().filter_map(|p| p.connects(cave))
    }

    fn paths(&self, special_cave: SpecialCave) -> Result<HashSet<Vec<&Cave<'a>>>, AocError> {
        // Paths from the given cave to the end having already visited some
        fn paths_rec<'b, 'c>(
            system: &'b CaveSystem<'c>,
            cave: &'b Cave<'c>,
            visited: &HashSet<&'b Cave<'c>>,
            mut special_cave: SpecialCave,
        ) -> Option<HashSet<Vec<&'b Cave<'c>>>> {
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
                if let SpecialCave::Some {cave: spec_cave, count } = special_cave && spec_cave == cave {
		    if count == 1 {
			visited.insert(cave);
		    } else {
			special_cave = SpecialCave::Some {cave: spec_cave, count: count - 1};
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

        paths_rec(self, &self.start, &HashSet::new(), special_cave)
            .ok_or_else(|| AocError::Process("No valid paths were found".into()))
    }

    fn paths_special(&self) -> Result<HashSet<Vec<&Cave<'a>>>, AocError> {
        let mut paths = HashSet::new();
        for cave in self.caves.iter().filter(|c| c.has_type(CaveType::Small)) {
            paths.extend(self.paths(SpecialCave::Some { cave, count: 2 })?)
        }
        Ok(paths)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 12,
    name: "Passage Pathing",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let cave_system = CaveSystem::from_str(input)?;
            let paths = cave_system.paths(SpecialCave::None)?;

            /*for path in paths.iter() {
                println!("{:?}", path);
            }
            println!();*/

            // Process
            Ok(Answer::Unsigned(paths.len().try_into().unwrap()))
        },
        // Part b)
        |input| {
            // Generation
            let cave_system = CaveSystem::from_str(input)?;
            let paths = cave_system.paths_special()?;

            // Process
            Ok(Answer::Unsigned(paths.len().try_into().unwrap()))
        },
    ],
};
