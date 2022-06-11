use nom::{combinator::map, sequence::tuple};
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::aoc::{parse::field_line_parser, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(1269), Unsigned(1309)],
    "Hit Points: 40
    Damage: 9",
    vec![734u64, 754].answer_vec()
    }
}

#[derive(new, Clone, Debug)]
struct Character {
    hit_points: u32,
    damage: u32,
    mana: u32,
    armor: u32,
    #[new(value = "Vec::new()")]
    spells: Vec<Box<dyn Spell>>,
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

    fn turn_cast(&mut self, spell_type: SpellType, opponent: &mut Character) -> bool {
        let spell = spell_type.create();

        // Apply effects
        self.apply_effects(opponent);
        opponent.apply_effects(self);

        // Check if this can be cast
        if self.dead() || spell.cost() > self.mana || self.spells.contains(&spell) {
            return false;
        }

        // Cast spell
        //println!("Casting {}!", spell.name());
        self.mana -= spell.cost();
        self.spells.push(spell);
        true
    }

    fn turn_attack(&mut self, opponent: &mut Character) {
        // Apply effects
        self.apply_effects(opponent);
        opponent.apply_effects(self);

        if !self.dead() {
            //println!("Attacks!");
            opponent.hurt(self.damage.saturating_sub(opponent.armor).max(1));
        }
    }

    fn apply_effects(&mut self, opponent: &mut Character) {
        let mut spells: Vec<Box<dyn Spell>> = self.spells.drain(..).collect();
        for spell in spells.iter_mut() {
            //spell.borrow_mut().apply_effect(self, opponent)
            spell.apply_effect(self, opponent)
        }
        self.spells
            .extend(spells.into_iter().filter(|s| !s.expired()))
    }
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
            )),
            |(hp, d)| Character::new(hp, d, 0, 0),
        )(input)
    }
}

trait Spell: SpellClone {
    fn new() -> Self
    where
        Self: Sized;
    fn name(&self) -> &'static str;
    fn cost(&self) -> u32;
    fn apply_effect(&mut self, player: &mut Character, opponent: &mut Character);
    fn expired(&self) -> bool;
}
impl PartialEq for dyn Spell {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}
impl fmt::Debug for dyn Spell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

// This is a workaround that allows these trait objects to be cloned.
// See: https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
trait SpellClone {
    fn clone_box(&self) -> Box<dyn Spell>;
}
impl<T> SpellClone for T
where
    T: 'static + Clone + Spell,
{
    fn clone_box(&self) -> Box<dyn Spell> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn Spell> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone)]
struct MagicMissile {
    used: bool,
}
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

    fn apply_effect(&mut self, _player: &mut Character, opponent: &mut Character) {
        if !self.used {
            opponent.hurt(4);
        }
        self.used = true;
    }

    fn expired(&self) -> bool {
        self.used
    }
}

#[derive(Clone)]
struct Drain {
    used: bool,
}
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

    fn apply_effect(&mut self, player: &mut Character, opponent: &mut Character) {
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

#[derive(Clone)]
struct Shield {
    turns: u8,
}
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

    fn apply_effect(&mut self, player: &mut Character, _opponent: &mut Character) {
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

#[derive(Clone)]
struct Poison {
    turns: u8,
}
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

    fn apply_effect(&mut self, _player: &mut Character, opponent: &mut Character) {
        if !self.expired() {
            opponent.hurt(3);
        }
        self.turns += 1;
    }

    fn expired(&self) -> bool {
        self.turns >= 6
    }
}

#[derive(Clone)]
struct Recharge {
    turn: u8,
}
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

    fn apply_effect(&mut self, player: &mut Character, _opponent: &mut Character) {
        if !self.expired() {
            player.mana += 101;
        }
        self.turn += 1;
    }

    fn expired(&self) -> bool {
        self.turn >= 5
    }
}

#[derive(Clone, Copy, EnumIter)]
enum SpellType {
    MagicMissile,
    Drain,
    Shield,
    Poison,
    Recharge,
}
impl SpellType {
    fn create(&self) -> Box<dyn Spell> {
        match *self {
            SpellType::MagicMissile => Box::new(MagicMissile::new()),
            SpellType::Drain => Box::new(Drain::new()),
            SpellType::Shield => Box::new(Shield::new()),
            SpellType::Poison => Box::new(Poison::new()),
            SpellType::Recharge => Box::new(Recharge::new()),
        }
    }
}

fn solve(player: Character, boss: Character, hard_mode: bool) -> AocResult<u64> {
    fn solve_rec(
        level: usize,
        spent: u32,
        min_spent: &mut Option<u32>,
        player: Character,
        boss: Character,
        hard_mode: bool,
    ) {
        let _indent = " ".repeat(level);

        // Create play branch with every spell cast on the player's turn
        for spell_type in SpellType::iter() {
            let mut player = player.clone();
            let mut boss = boss.clone();
            if hard_mode {
                player.hurt(1)
            }
            if !player.dead() && player.turn_cast(spell_type, &mut boss) {
                let spell = spell_type.create();
                let spent = spent + spell.cost();
                //println!("{}Casted {}", _indent, spell.name());
                // Abandon branch if the player has already spent too much
                if let Some(s) = min_spent {
                    if spent > *s {
                        //println!("{}Abandoning branch!", _indent);
                        continue;
                    }
                }

                boss.turn_attack(&mut player);

                if boss.dead() {
                    //println!("{}Boss killed with {} mana spent!", _indent, spent);
                    *min_spent = match min_spent {
                        None => Some(spent),
                        Some(s) => Some(spent.min(*s)),
                    }
                } else if player.dead() {
                    //println!("{}Player was killed!", _indent);
                } else {
                    solve_rec(level + 1, spent, min_spent, player, boss, hard_mode);
                }
            }
        }
    }

    let mut min_spent = None;
    solve_rec(0, 0, &mut min_spent, player, boss, hard_mode);
    Ok(min_spent
        .ok_or_else(|| AocError::Process("The boss always wins!".into()))?
        .into())
}

pub const SOLUTION: Solution = Solution {
    day: 22,
    name: "Wizard Simulator 20XX",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let boss = Character::from_str(input.expect_input()?)?;
            let player = Character::new(50, 0, 500, 0);

            // Test for example battle
            /*let mut player = Character::new(10, 0, 250, 0);
            let mut boss = Character::new(14, 8, 0, 0);
            use SpellType::*;
            for spell in [Recharge, Shield, Drain, Poison, MagicMissile] {
                println!("\nPlayer turn:");
                println!("Player: {:?}", player);
                println!("Boss: {:?}", boss);
                player.turn_cast(spell, &mut boss);

                println!("\nBoss turn:");
                println!("Player: {:?}", player);
                println!("Boss: {:?}", boss);
                boss.turn_attack(&mut player);
            }*/

            // Process
            Ok(solve(player, boss, false)?.into())
        },
        // Part b)
        |input| {
            // Generation
            let boss = Character::from_str(input.expect_input()?)?;
            let player = Character::new(50, 0, 500, 0);

            // Process
            Ok(solve(player, boss, true)?.into())
        },
    ],
};
