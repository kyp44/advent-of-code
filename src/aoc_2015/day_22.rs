use nom::{combinator::map, sequence::tuple};

use crate::aoc::{parse::field_line_parser, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![],
    "Hit Points: 58
    Damage: 9",
    vec![1u64].answer_vec()
    }
}

#[derive(new, Debug)]
struct Character {
    hit_points: u32,
    damage: u32,
    mana: u32,
    armor: u32,
}
impl Character {
    fn heal(&mut self, amount: u32) {
        self.hit_points += amount;
    }

    fn hurt(&mut self, damage: u32) {
        self.hit_points = self.hit_points.saturating_sub(damage);
    }

    fn dead(&self) -> bool {
        self.hit_points == 0
    }
}
impl Parseable<'_> for Character {
    fn parser(input: &str) -> NomParseResult<Self>
    where
        Self: Sized,
    {
        map(
            tuple((
                field_line_parser("Hit Points:", nom::character::complete::u32),
                field_line_parser("Damage:", nom::character::complete::u32),
            )),
            |(hp, d)| Character::new(hp, d, 0, 0),
        )(input)
    }
}

trait Spell {
    fn new() -> Self;
    fn name(&self) -> &'static str;
    fn cost(&self) -> u32;
    fn turn(&mut self, player: &mut Character, opponent: &mut Character);
    fn expired(&self) -> bool;
}
macro_rules! spell_eq {
    ($type: ident) => {
        impl PartialEq for $type {
            fn eq(&self, other: &Self) -> bool {
                self.name() == other.name()
            }
        }
        impl Eq for $type {}
    };
}

struct MagicMissile {
    used: bool,
}
spell_eq! {MagicMissile}
impl Spell for MagicMissile {
    fn new() -> Self {
        MagicMissile { used: false }
    }

    fn name(&self) -> &'static str {
        "Magic Missile"
    }

    fn cost(&self) -> u32 {
        53
    }

    fn turn(&mut self, _player: &mut Character, opponent: &mut Character) {
        if !self.used {
            opponent.hurt(4);
        }
        self.used = true;
    }

    fn expired(&self) -> bool {
        self.used
    }
}

struct Drain {
    used: bool,
}
spell_eq! {Drain}
impl Spell for Drain {
    fn new() -> Self {
        Drain { used: false }
    }

    fn name(&self) -> &'static str {
        "Drain"
    }

    fn cost(&self) -> u32 {
        73
    }

    fn turn(&mut self, player: &mut Character, opponent: &mut Character) {
        if !self.used {
            opponent.hurt(2);
            player.heal(2);
        }
        self.used = true;
    }

    fn expired(&self) -> bool {
        self.used
    }
}

struct Shield {
    turns: u8,
}
spell_eq! {Shield}
impl Spell for Shield {
    fn new() -> Self {
        Shield { turns: 0 }
    }

    fn name(&self) -> &'static str {
        "Shield"
    }

    fn cost(&self) -> u32 {
        113
    }

    fn turn(&mut self, player: &mut Character, opponent: &mut Character) {
        if self.turns == 0 {
            player.armor += 7;
        }
        self.turns += 1;
        if self.expired() {
            player.armor = player.armor.saturating_sub(7)
        }
    }

    fn expired(&self) -> bool {
        self.turns >= 6
    }
}

struct Poison {
    turns: u8,
}
spell_eq! {Poison}
impl Spell for Poison {
    fn new() -> Self {
        Poison { turns: 0 }
    }

    fn name(&self) -> &'static str {
        "Poison"
    }

    fn cost(&self) -> u32 {
        173
    }

    fn turn(&mut self, player: &mut Character, opponent: &mut Character) {
        if !self.expired() {
            opponent.hurt(3);
        }
        self.turns += 1;
    }

    fn expired(&self) -> bool {
        self.turns >= 6
    }
}

struct Recharge {
    turn: u8,
}
spell_eq! {Recharge}
impl Spell for Recharge {
    fn new() -> Self {
        Recharge { turn: 0 }
    }

    fn name(&self) -> &'static str {
        "Recharge"
    }

    fn cost(&self) -> u32 {
        229
    }

    fn turn(&mut self, player: &mut Character, opponent: &mut Character) {
        if !self.expired() {
            player.mana += 101;
        }
        self.turn += 1;
    }

    fn expired(&self) -> bool {
        self.turn >= 5
    }
}

pub const SOLUTION: Solution = Solution {
    day: 22,
    name: "Wizard Simulator 20XX",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let boss = Character::from_str(input)?;

            println!("TODO: {:?}", boss);

            // Process
            Ok(0u64.into())
        },
    ],
};
