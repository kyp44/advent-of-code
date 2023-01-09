use crate::aoc::prelude::*;

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

/// Contains solution implementation items.
mod solution {
    use crate::aoc::parse::field_line_parser;

    use super::*;
    use nom::{combinator::map, sequence::tuple};
    use std::fmt;
    use strum::IntoEnumIterator;
    use strum_macros::EnumIter;

    /// The player or the boss character, which can be parsed from text input.
    #[derive(new, Clone, Debug)]
    pub struct Character {
        /// Current number of hit points.
        hit_points: u32,
        /// Damage dealt by physical attacks.
        damage: u32,
        /// Current mana remaining.
        mana: u32,
        /// Armer stat.
        armor: u32,
        /// Active spells affecting the character.
        #[new(value = "Vec::new()")]
        spells: Vec<Box<dyn Spell>>,
    }
    impl Character {
        /// Heals the charater by some number of hit points.
        fn heal(&mut self, amount: u32) {
            self.hit_points += amount;
        }

        /// Hurt the character by some number of hit points.
        fn hurt(&mut self, damage: u32) {
            self.hit_points = self.hit_points.saturating_sub(damage);
        }

        /// Whether or not the character is currently dead.
        fn dead(&self) -> bool {
            self.hit_points == 0
        }

        /// Take a turn in which this characer casts a spell against an opponent.
        /// Returns whether or not the spell could be and was cast.
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

        /// Take a turn in which the character performs a physical attack against
        /// an opponent.
        fn turn_attack(&mut self, opponent: &mut Character) {
            // Apply effects
            self.apply_effects(opponent);
            opponent.apply_effects(self);

            if !self.dead() {
                //println!("Attacks!");
                opponent.hurt(self.damage.saturating_sub(opponent.armor).max(1));
            }
        }

        /// Apply the effects of any active spell both on this character as well
        /// as on an opponent.
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

    /// Interface for spells.
    trait Spell: SpellClone {
        /// Create the spell.
        fn new() -> Self
        where
            Self: Sized;
        /// Name of the spell.
        fn name(&self) -> &'static str;
        /// Mana cost to cast the spell.
        fn cost(&self) -> u32;
        /// Apply the effects of the spell on the player and boss characters.
        fn apply_effect(&mut self, player: &mut Character, opponent: &mut Character);
        /// Whether or not the spell has expired given the number of times its affect has been applied.
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

    /// This is a workaround that allows [Spell] trait objects to be cloned.
    /// See [this StackOverflow post](https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object).
    trait SpellClone {
        /// Clone the trait object.
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

    /// The magic missile spell.
    #[derive(Clone)]
    struct MagicMissile {
        /// Whether te spell has been used (i.e. applied).
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

    /// Drain spell.
    #[derive(Clone)]
    struct Drain {
        /// Whether or not the spell has been used (i.e. applied).
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

    /// Shield effect spell.
    #[derive(Clone)]
    struct Shield {
        /// Number of turns/times the effect has been applied.
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

    /// Poison spell.
    #[derive(Clone)]
    struct Poison {
        /// Number of turns/times the effect has been applied.
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

    /// Mana recharge spell.
    #[derive(Clone)]
    struct Recharge {
        /// Number of turns/times the effect has been applied.
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

    /// All possible spells.
    #[derive(Clone, Copy, EnumIter)]
    enum SpellType {
        /// Magic missile spell.
        MagicMissile,
        /// Drain spell.
        Drain,
        /// Shield spell.
        Shield,
        /// Poison spell.
        Poison,
        /// Mana recharge spell.
        Recharge,
    }
    impl SpellType {
        /// Create the spell of this type.
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

    /// Solves a part of the problem by playing games and recursively having the player
    /// try every combination of spells as the turns progress.
    /// `hard_mode` causes the player character to take 1 damage at the beginning of
    /// each player turn.
    pub fn solve(player: Character, boss: Character, hard_mode: bool) -> AocResult<u64> {
        /// Recursive sub-function of [solve].
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
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 22,
    name: "Wizard Simulator 20XX",
    preprocessor: None,
    solvers: &[
        // Part one
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
        // Part two
        |input| {
            // Generation
            let boss = Character::from_str(input.expect_input()?)?;
            let player = Character::new(50, 0, 500, 0);

            // Process
            Ok(solve(player, boss, true)?.into())
        },
    ],
};
