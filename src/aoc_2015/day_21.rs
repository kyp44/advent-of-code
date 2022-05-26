use std::{iter::Sum, ops::Add};

use itertools::{iproduct, Itertools, MinMaxResult};
use nom::{combinator::map, sequence::tuple};

use crate::aoc::{parse::field_line_parser, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(111), Unsigned(188)],
    "Hit Points: 90
Damage: 3
Armor: 3",
    vec![23u64, 33].answer_vec()
    }
}

#[derive(Debug, new)]
struct Stats {
    damage: u32,
    armor: u32,
}
impl Add for &Stats {
    type Output = Stats;

    fn add(self, rhs: Self) -> Self::Output {
        Stats::new(self.damage + rhs.damage, self.armor + rhs.armor)
    }
}
impl<'a> Sum<&'a Stats> for Stats {
    fn sum<I: Iterator<Item = &'a Stats>>(iter: I) -> Self {
        let mut acc = Stats::new(0, 0);
        for stats in iter {
            acc = &acc + stats;
        }
        acc
    }
}

#[derive(Debug)]
struct ShopItem {
    _name: &'static str,
    cost: u32,
    stats: Stats,
}
macro_rules! shop_item {
    ($name:literal, $cost: expr, $damage: expr, $armor: expr) => {
        ShopItem {
            _name: $name,
            cost: $cost,
            stats: Stats {
                damage: $damage,
                armor: $armor,
            },
        }
    };
}

const WEAPONS: &[ShopItem] = &[
    shop_item!("Dagger", 8, 4, 0),
    shop_item!("Shortsword", 10, 5, 0),
    shop_item!("Warhammer", 25, 6, 0),
    shop_item!("Longsword", 40, 7, 0),
    shop_item!("Greataxe", 74, 8, 0),
];

const ARMORS: &[ShopItem] = &[
    shop_item!("Leather", 13, 0, 1),
    shop_item!("Chainmail", 31, 0, 2),
    shop_item!("Splintmail", 53, 0, 3),
    shop_item!("Bandedmail", 75, 0, 4),
    shop_item!("Platemail", 102, 0, 5),
];

const RINGS: &[ShopItem] = &[
    shop_item!("Damage +1", 25, 1, 0),
    shop_item!("Damage +2", 50, 2, 0),
    shop_item!("Damage +3", 100, 3, 0),
    shop_item!("Defense +1", 20, 0, 1),
    shop_item!("Defense +2", 40, 0, 2),
    shop_item!("Defense +3", 80, 0, 3),
];

#[derive(Debug, new)]
struct Character {
    hit_points: u32,
    stats: Stats,
}
impl Parseable<'_> for Character {
    fn parser(input: &str) -> NomParseResult<&str, Self>
    where
        Self: Sized,
    {
        map(
            tuple((
                field_line_parser("Hit Points:", nom::character::complete::u32),
                field_line_parser("Damage:", nom::character::complete::u32),
                field_line_parser("Armor:", nom::character::complete::u32),
            )),
            |(hp, d, a)| Character::new(hp, Stats::new(d, a)),
        )(input)
    }
}
impl Character {
    fn battle(&self, other: &Self) -> bool {
        let mut hp = self.hit_points;
        let mut hpo = other.hit_points;

        fn turn(_astr: &str, _bstr: &str, a: &Character, b: &Character, hp: &mut u32) -> bool {
            let damage = a.attack(b);
            *hp = hp.saturating_sub(damage);
            /*println!(
                "{} does {} damage; {} goes down to {} HP",
                _astr, damage, _bstr, hp
            );*/
            *hp < 1
        }

        loop {
            // Self turn
            if turn("Self", "Other", self, other, &mut hpo) {
                return true;
            }

            // Other's turn
            if turn("Other", "Self", other, self, &mut hp) {
                return false;
            }
        }
    }

    fn attack(&self, other: &Self) -> u32 {
        self.stats.damage.saturating_sub(other.stats.armor).max(1)
    }
}

trait Part {
    fn select_cost(&self, min_max: &(u32, u32)) -> u32;
    fn win_or_lose(&self, won: bool) -> bool;
}
struct PartA;
impl Part for PartA {
    fn select_cost(&self, min_max: &(u32, u32)) -> u32 {
        min_max.0
    }

    fn win_or_lose(&self, won: bool) -> bool {
        won
    }
}
struct PartB;
impl Part for PartB {
    fn select_cost(&self, min_max: &(u32, u32)) -> u32 {
        min_max.1
    }

    fn win_or_lose(&self, won: bool) -> bool {
        !won
    }
}

#[derive(new)]
struct Problem {
    boss: Character,
}
impl Problem {
    fn solve(&self, part: &dyn Part) -> AocResult<u64> {
        // Go through every combination of 1 weapon, 0-1 armor, and 0-2 rings
        match iproduct!(
            WEAPONS.iter(),
            ARMORS.iter().none_iter(),
            (0..=2).flat_map(|n| RINGS.iter().combinations(n))
        )
        .filter_map(|(weapon, armor, rings)| {
            let equipment = {
                let mut v = vec![weapon];
                if let Some(item) = armor {
                    v.push(item);
                }
                v.extend(rings);
                v
            };

            //println!("{}", equipment.iter().map(|item| item._name).join(", "));
            let cost: u32 = equipment.iter().map(|item| item.cost).sum();
            let player = Character::new(100, equipment.into_iter().map(|item| &item.stats).sum());

            if part.win_or_lose(player.battle(&self.boss)) {
                Some(cost)
            } else {
                None
            }
        })
        .minmax()
        {
            MinMaxResult::NoElements => Err(AocError::Process("The player can never win!".into())),
            MinMaxResult::OneElement(m) => Ok(m.into()),
            MinMaxResult::MinMax(min, max) => Ok(part.select_cost(&(min, max)).into()),
        }
    }
}

pub const SOLUTION: Solution = Solution {
    day: 21,
    name: "RPG Simulator 20XX",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let problem = Problem::new(Character::from_str(input)?);

            // Just a test for the example battle
            /*let player = Character::new(8, Stats::new(5, 5));
            let boss = Character::new(12, Stats::new(7, 2));
            player.battle(&boss);*/

            // Process
            Ok(problem.solve(&PartA)?.into())
        },
        // Part b)
        |input| {
            // Generation
            let problem = Problem::new(Character::from_str(input)?);

            // Process
            Ok(problem.solve(&PartB)?.into())
        },
    ],
};
