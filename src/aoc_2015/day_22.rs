use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "Hit Points: 40
    Damage: 9";
            answers = unsigned![734, 754];
        }
        actual_answers = unsigned![1269, 1309];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{
        parse::field_line_parser,
        tree_search::{BestMetricAction, BestMetricTreeNode, Metric, MetricChild},
    };
    use derive_more::Add;
    use derive_new::new;
    use enum_dispatch::enum_dispatch;
    use infinitable::Infinitable;
    use nom::{combinator::map, sequence::tuple};
    use std::{fmt, hash::Hash};

    /// Interface for spells.
    #[enum_dispatch]
    trait SpellEffect: Clone + Eq + Hash {
        /// Returns the name of the spell.
        fn name(&self) -> &'static str;
        /// Returns the mana cost to cast the spell.
        fn cost(&self) -> u32;
        /// Applies the effects of the spell on the player and boss characters.
        fn apply_effect(&mut self, player: &mut Character, opponent: &mut Character);
        /// Returns the whether or not the spell has expired given the number of
        /// times its affect has been applied.
        fn expired(&self) -> bool;
    }

    /// The magic missile spell.
    #[derive(Clone, Default, PartialEq, Eq, Hash)]
    struct MagicMissile {
        /// Whether the spell has been used (i.e. applied).
        used: bool,
    }
    impl SpellEffect for MagicMissile {
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
    #[derive(Clone, Default, PartialEq, Eq, Hash)]
    struct Drain {
        /// Whether or not the spell has been used (i.e. applied).
        used: bool,
    }
    impl SpellEffect for Drain {
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
    #[derive(Clone, Default, PartialEq, Eq, Hash)]
    struct Shield {
        /// Number of turns/times the effect has been applied.
        turns: u8,
    }
    impl SpellEffect for Shield {
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
    #[derive(Clone, Default, PartialEq, Eq, Hash)]
    struct Poison {
        /// Number of turns/times the effect has been applied.
        turns: u8,
    }
    impl SpellEffect for Poison {
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
    #[derive(Clone, Default, PartialEq, Eq, Hash)]
    struct Recharge {
        /// Number of turns/times the effect has been applied.
        turns: u8,
    }
    impl SpellEffect for Recharge {
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
            self.turns += 1;
        }

        fn expired(&self) -> bool {
            self.turns >= 5
        }
    }

    /// Possible spells the player can cast.
    #[enum_dispatch(SpellEffect)]
    #[derive(Clone, PartialEq, Eq, Hash)]
    enum Spell {
        MagicMissile,
        Drain,
        Shield,
        Poison,
        Recharge,
    }
    impl fmt::Debug for Spell {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name())
        }
    }
    impl Spell {
        /// Returns an iterator over all spells in their default state.
        pub fn iter() -> impl Iterator<Item = Self> {
            [
                MagicMissile::default().into(),
                Drain::default().into(),
                Shield::default().into(),
                Poison::default().into(),
                Recharge::default().into(),
            ]
            .into_iter()
        }
    }

    /// The player or the boss character, which can be parsed from text input.
    #[derive(new, Clone, PartialEq, Eq, Hash, Debug)]
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
        spells: Vec<Spell>,
    }
    impl Character {
        /// Heals the character by some number of hit points.
        fn heal(&mut self, amount: u32) {
            self.hit_points += amount;
        }

        /// Hurts the character by some number of hit points.
        fn hurt(&mut self, damage: u32) {
            self.hit_points = self.hit_points.saturating_sub(damage);
        }

        /// Returns whether or not the character is currently dead.
        fn dead(&self) -> bool {
            self.hit_points == 0
        }

        /// Takes a turn in which this character casts a spell against an opponent.
        ///
        /// Returns whether or not the spell could be and was cast.
        fn turn_cast(&mut self, spell: Spell, opponent: &mut Character) -> bool {
            // Apply effects
            self.apply_effects(opponent);
            opponent.apply_effects(self);

            // Check if this can be cast
            if self.dead()
                || spell.cost() > self.mana
                || self.spells.iter().any(|s| s.name() == spell.name())
            {
                return false;
            }

            // Cast spell
            //println!("Casting {}!", spell.name());
            self.mana -= spell.cost();
            self.spells.push(spell);
            true
        }

        /// Takes a turn in which the character performs a physical attack against
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

        /// Applies the effects of any active spell both on this character as well
        /// as on an opponent.
        fn apply_effects(&mut self, opponent: &mut Character) {
            let mut spells: Vec<Spell> = self.spells.drain(..).collect();
            for spell in spells.iter_mut() {
                spell.apply_effect(self, opponent)
            }
            self.spells
                .extend(spells.into_iter().filter(|s| !s.expired()))
        }
    }
    impl Parsable<'_> for Character {
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

    /// The characters involved in a battle.
    #[derive(Clone, Debug, new, PartialEq, Eq, Hash)]
    pub struct Characters {
        /// Whether or not we are in hard mode (Part two).
        #[new(value = "false")]
        hard_mode: bool,
        /// The player.
        player: Character,
        /// The boss.
        boss: Character,
    }
    impl Characters {
        /// Searches the game tree to determine the minimal mana cost in which the player wins.
        pub fn minimal_mana_cost(mut self, hard_mode: bool) -> AocResult<u64> {
            self.hard_mode = hard_mode;
            match self.best_metric().0 {
                Infinitable::Finite(m) => Ok(m.into()),
                _ => Err(AocError::NoSolution),
            }
        }
    }

    /// Relative or cumulative mana cost for spells.
    #[derive(Clone, Copy, Debug, Add)]
    pub struct Mana(Infinitable<u32>);
    impl Metric for Mana {
        const INITIAL_BEST: Self = Mana(Infinitable::Infinity);
        const INITIAL_COST: Self = Mana(Infinitable::Finite(0));

        fn is_better(&self, other: &Self) -> bool {
            self.0 < other.0
        }
    }
    impl From<u32> for Mana {
        fn from(value: u32) -> Self {
            Self(value.into())
        }
    }
    impl BestMetricTreeNode for Characters {
        type Metric = Mana;

        fn recurse_action(&self, _cumulative_cost: &Self::Metric) -> BestMetricAction<Self> {
            // Only count victory if the boss is dead
            if self.boss.dead() {
                return BestMetricAction::StopSuccess;
            }

            let mut player = self.player.clone();

            // If in hard mode the player takes damage no matter.
            if self.hard_mode {
                player.hurt(1)
            }

            // If the player is dead than we are done and we lost.
            if player.dead() {
                return BestMetricAction::StopFailure;
            }

            BestMetricAction::Continue(
                Spell::iter()
                    .filter_map(|spell| {
                        let mut player = player.clone();
                        let mut boss = self.boss.clone();
                        let cost = spell.cost();

                        if player.turn_cast(spell, &mut boss) {
                            boss.turn_attack(&mut player);

                            Some(MetricChild::new(
                                Characters {
                                    hard_mode: self.hard_mode,
                                    player,
                                    boss,
                                },
                                cost.into(),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect(),
            )
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 22,
    name: "Wizard Simulator 20XX",
    preprocessor: Some(|input| {
        Ok(Box::new(Characters::new(
            Character::new(50, 0, 500, 0),
            Character::from_str(input)?,
        ))
        .into())
    }),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Characters>()?
                .clone()
                .minimal_mana_cost(false)?
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Characters>()?
                .clone()
                .minimal_mana_cost(true)?
                .into())
        },
    ],
};
