use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(111), Unsigned(188)],
    "Hit Points: 90
Damage: 3
Armor: 3",
    vec![23u64, 33].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::field_line_parser;
    use derive_new::new;
    use itertools::{iproduct, Itertools, MinMaxResult};
    use nom::{combinator::map, sequence::tuple};
    use std::{iter::Sum, ops::Add};

    /// Stats for a character.
    #[derive(Debug, new)]
    pub struct Stats {
        /// Damage dealt.
        damage: u32,
        /// Armor.
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

    /// An item sold by the shopkeeper.
    #[derive(Debug)]
    struct ShopItem {
        /// Name of the item.
        _name: &'static str,
        /// Cost to buy the item.
        cost: u32,
        /// Stat boost imparted by the item.
        stats: Stats,
    }

    /// Macro to more conveniently define a shop item literal.
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

    /// Weapons available at the shop.
    const WEAPONS: &[ShopItem] = &[
        shop_item!("Dagger", 8, 4, 0),
        shop_item!("Shortsword", 10, 5, 0),
        shop_item!("Warhammer", 25, 6, 0),
        shop_item!("Longsword", 40, 7, 0),
        shop_item!("Greataxe", 74, 8, 0),
    ];

    /// Armor avilable at the shop.
    const ARMOR: &[ShopItem] = &[
        shop_item!("Leather", 13, 0, 1),
        shop_item!("Chainmail", 31, 0, 2),
        shop_item!("Splintmail", 53, 0, 3),
        shop_item!("Bandedmail", 75, 0, 4),
        shop_item!("Platemail", 102, 0, 5),
    ];

    /// Rings available at the shop.
    const RINGS: &[ShopItem] = &[
        shop_item!("Damage +1", 25, 1, 0),
        shop_item!("Damage +2", 50, 2, 0),
        shop_item!("Damage +3", 100, 3, 0),
        shop_item!("Defense +1", 20, 0, 1),
        shop_item!("Defense +2", 40, 0, 2),
        shop_item!("Defense +3", 80, 0, 3),
    ];

    /// A character, which is either the player or the boss.
    ///
    /// This can be parsed from text input.
    #[derive(Debug, new)]
    pub struct Character {
        /// Total hit points.
        hit_points: u32,
        /// Starting stats.
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
        /// Have a battle with another character until one runs out of hit points and dies.
        ///
        /// Returns whether this character won the battle.
        fn battle(&self, other: &Self) -> bool {
            let mut hp = self.hit_points;
            let mut hpo = other.hit_points;

            /// Take character `a`'s turn, attacking player `b`, who has a specified number of hit points.
            ///
            /// Returns whether the attack killed player `b`.
            /// This is an internal function of [`Character::battle`].
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

        /// Attacks another character, returning the total damage dealt.
        fn attack(&self, other: &Self) -> u32 {
            self.stats.damage.saturating_sub(other.stats.armor).max(1)
        }
    }

    /// Behavior specific to one particular part of the problem.
    pub trait Part {
        /// Given two costs, select the appropriate one, based on whether we are
        /// minimizing or maximizing.
        fn select_cost(min_max: &(u32, u32)) -> u32;
        /// Given whether the player won or not, returns whether this is what we
        /// want or not.
        fn win_or_lose(won: bool) -> bool;
    }

    /// Behavior for part one.
    pub struct PartOne;
    impl Part for PartOne {
        fn select_cost(min_max: &(u32, u32)) -> u32 {
            min_max.0
        }

        fn win_or_lose(won: bool) -> bool {
            won
        }
    }

    /// Behavior for part two.
    pub struct PartTwo;
    impl Part for PartTwo {
        fn select_cost(min_max: &(u32, u32)) -> u32 {
            min_max.1
        }

        fn win_or_lose(won: bool) -> bool {
            !won
        }
    }

    /// Defines the problem, which can be parsed from text input.
    #[derive(new)]
    pub struct Problem {
        /// The boss character and his stats.
        boss: Character,
    }
    impl Problem {
        /// Solves a part of the problem by playing out the game for every combination
        /// of the allowed load out bought from the shop.
        pub fn solve<P: Part>(&self) -> AocResult<u64> {
            // Go through every combination of 1 weapon, 0-1 armor, and 0-2 rings
            match iproduct!(
                WEAPONS.iter(),
                [None].into_iter().chain(ARMOR.iter().map(Some)),
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
                let player =
                    Character::new(100, equipment.into_iter().map(|item| &item.stats).sum());

                if P::win_or_lose(player.battle(&self.boss)) {
                    Some(cost)
                } else {
                    None
                }
            })
            .minmax()
            {
                MinMaxResult::NoElements => {
                    Err(AocError::Process("The player can never win!".into()))
                }
                MinMaxResult::OneElement(m) => Ok(m.into()),
                MinMaxResult::MinMax(min, max) => Ok(P::select_cost(&(min, max)).into()),
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 21,
    name: "RPG Simulator 20XX",
    preprocessor: Some(|input| Ok(Box::new(Problem::new(Character::from_str(input)?)).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<Problem>()?.solve::<PartOne>()?.into())
        },
        // Part two
        |input| {
            // Process
            Ok(input.expect_data::<Problem>()?.solve::<PartTwo>()?.into())
        },
    ],
};
