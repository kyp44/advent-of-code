use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![],
    "Hit Points: 109
Damage: 8
Armor: 2",
    vec![0u64].answer_vec()
    }
}

#[derive(Debug)]
struct Stats {
    damage: u32,
    armor: u32,
}

#[derive(Debug)]
struct ShopItem {
    name: &'static str,
    cost: u32,
    stats: Stats,
}
macro_rules! shop_item {
    ($name:literal, $cost: expr, $damage: expr, $armor: expr) => {
        ShopItem {
            name: $name,
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

fn test() {
    for weapon in WEAPONS.iter().none_iter() {
        println!("{:?}", weapon);
    }
}

pub const SOLUTION: Solution = Solution {
    day: 21,
    name: "RPG Simulator 20XX",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            test();

            // Process
            Ok(1u64.into())
        },
    ],
};
