use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(222870), Unsigned(117936)],
    "Butterscotch: capacity -1, durability -2, flavor 6, texture 3, calories 8
    Cinnamon: capacity 2, durability 3, flavor -2, texture -1, calories 3",
    vec![62842880u64, 57600000].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use nom::{
        bytes::complete::{tag, take_until},
        combinator::map,
        sequence::tuple,
    };
    use std::{
        convert::TryInto,
        ops::{Add, Mul},
        str::FromStr,
    };

    /// A cookie ingredient with its properties, which can be parsed from text input.
    #[derive(Debug)]
    pub struct Ingredient {
        /// How well the cookie absorbs milk.
        capacity: i64,
        /// How well the cookie stays intact when full of milk.
        durability: i64,
        /// How tasty it makes the cookie.
        flavor: i64,
        /// How well it improves the feel of the cookie.
        texture: i64,
        /// How many calories it adds to the cookie.
        calories: i64,
    }
    impl Parseable<'_> for Ingredient {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                tuple((
                    take_until(":"),
                    tag(": capacity "),
                    nom::character::complete::i64,
                    tag(", durability "),
                    nom::character::complete::i64,
                    tag(", flavor "),
                    nom::character::complete::i64,
                    tag(", texture "),
                    nom::character::complete::i64,
                    tag(", calories "),
                    nom::character::complete::i64,
                )),
                |(_, _, capacity, _, durability, _, flavor, _, texture, _, calories)| Ingredient {
                    capacity,
                    durability,
                    flavor,
                    texture,
                    calories,
                },
            )(input.trim())
        }
    }
    impl Add for Ingredient {
        type Output = Ingredient;

        fn add(self, rhs: Self) -> Self::Output {
            &self + &rhs
        }
    }
    impl Add for &Ingredient {
        type Output = Ingredient;

        fn add(self, rhs: Self) -> Self::Output {
            Ingredient {
                capacity: self.capacity + rhs.capacity,
                durability: self.durability + rhs.durability,
                flavor: self.flavor + rhs.flavor,
                texture: self.texture + rhs.texture,
                calories: self.calories + rhs.calories,
            }
        }
    }
    impl Mul<i64> for &Ingredient {
        type Output = Ingredient;

        fn mul(self, rhs: i64) -> Self::Output {
            Ingredient {
                capacity: self.capacity * rhs,
                durability: self.durability * rhs,
                flavor: self.flavor * rhs,
                texture: self.texture * rhs,
                calories: self.calories * rhs,
            }
        }
    }
    impl Ingredient {
        /// The total score of a total ingredient.
        fn score(&self) -> u64 {
            if self.capacity < 0 || self.durability < 0 || self.flavor < 0 || self.texture < 0 {
                return 0;
            }
            (self.capacity * self.durability * self.flavor * self.texture)
                .try_into()
                .unwrap()
        }
    }

    /// Recursively [Iterator] over all permutations that sum to a particular value
    #[derive(new)]
    struct SumPermutations<T> {
        /// TODO
        sum: T,
        bins: usize,
        #[new(value = "None")]
        i: Option<T>,
        #[new(value = "None")]
        sub: Option<Box<SumPermutations<T>>>,
    }
    impl<T> Iterator for SumPermutations<T>
    where
        T: Copy + num::Num + PartialOrd,
    {
        type Item = Vec<T>;

        fn next(&mut self) -> Option<Self::Item> {
            // Base cases
            if self.bins == 0 {
                return None;
            } else if self.bins == 1 {
                return match self.i {
                    Some(_) => None,
                    None => {
                        self.i = Some(T::zero());
                        Some(vec![self.sum])
                    }
                };
            }

            if self.i.is_none() {
                self.i = Some(T::zero());
            }
            let i = self.i.unwrap();
            if self.sub.is_none() {
                if i > self.sum {
                    return None;
                }
                self.sub = Some(Box::new(SumPermutations::new(self.sum - i, self.bins - 1)))
            }
            match self.sub.as_mut().unwrap().next() {
                Some(mut sv) => {
                    sv.insert(0, i);
                    Some(sv)
                }
                None => {
                    self.i = Some(i + T::one());
                    self.sub = None;
                    self.next()
                }
            }
        }
    }

    /// Behavior specific to a particular problem part
    pub trait Part {
        /// Determines if a cookie with partcuilar total ingredients is valid to consider for the part.
        fn valid_recipe(_ingredient: &Ingredient) -> bool {
            true
        }
    }

    /// Behavior for part one.
    pub struct PartOne;
    impl Part for PartOne {}

    /// Behavior for part two.
    pub struct PartTwo;
    impl Part for PartTwo {
        fn valid_recipe(ingredient: &Ingredient) -> bool {
            ingredient.calories == 500
        }
    }

    /// Problem definition that can be parsed from text input.
    #[derive(Debug)]
    pub struct Problem {
        /// List of ingredients in our kitchen.
        ingredients: Box<[Ingredient]>,
    }
    impl FromStr for Problem {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Problem {
                ingredients: Ingredient::gather(s.lines())?.into_boxed_slice(),
            })
        }
    }
    impl Problem {
        /// Determine score of the highest scoring cookie possible.
        pub fn best_recipe<P: Part>(&self) -> u64 {
            SumPermutations::new(100, self.ingredients.len())
                .map(|amounts| {
                    amounts
                        .into_iter()
                        .zip(self.ingredients.iter())
                        .map(|(a, ing)| ing * a)
                        .reduce(|a, b| a + b)
                        .unwrap()
                })
                .filter(|ing| P::valid_recipe(ing))
                .map(|ing| ing.score())
                .max()
                .unwrap()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 15,
    name: "Science for Hungry People",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let problem: Problem = input.expect_input()?.parse()?;

            // Process
            Ok(problem.best_recipe::<PartOne>().into())
        },
        // Part two
        |input| {
            // Generation
            let problem: Problem = input.expect_input()?.parse()?;

            // Process
            Ok(problem.best_recipe::<PartTwo>().into())
        },
    ],
};
