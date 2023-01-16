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
        iter::Step,
        ops::{Add, Mul, RangeInclusive},
        str::FromStr,
    };

    #[cfg(test)]
    mod tests {
        use super::SumPermutations;

        #[test]
        fn sum_permutations() {
            let perms: Vec<Vec<u8>> = SumPermutations::new(6, 3).collect();

            assert_eq!(
                perms,
                vec![
                    vec![0, 0, 6],
                    vec![0, 1, 5],
                    vec![0, 2, 4],
                    vec![0, 3, 3],
                    vec![0, 4, 2],
                    vec![0, 5, 1],
                    vec![0, 6, 0],
                    vec![1, 0, 5],
                    vec![1, 1, 4],
                    vec![1, 2, 3],
                    vec![1, 3, 2],
                    vec![1, 4, 1],
                    vec![1, 5, 0],
                    vec![2, 0, 4],
                    vec![2, 1, 3],
                    vec![2, 2, 2],
                    vec![2, 3, 1],
                    vec![2, 4, 0],
                    vec![3, 0, 3],
                    vec![3, 1, 2],
                    vec![3, 2, 1],
                    vec![3, 3, 0],
                    vec![4, 0, 2],
                    vec![4, 1, 1],
                    vec![4, 2, 0],
                    vec![5, 0, 1],
                    vec![5, 1, 0],
                    vec![6, 0, 0],
                ]
            );
        }
    }

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

    /// Recursive [Iterator] over all permutations of some number of numeric values that sum to a constant.
    struct SumPermutations<T> {
        /// Number to which the permutations must sum.
        sum: T,
        /// Number of elements in each permutation.
        number: usize,
        /// Iterator that goes through the possible first elements.
        first_iter: RangeInclusive<T>,
        /// Current first element.
        first: T,
        /// Iterator over the sum permutations for the remaining elements after the first one.
        rest_iter: Box<dyn Iterator<Item = Vec<T>>>,
    }
    impl<T: Copy + Step + num::Num + 'static> SumPermutations<T> {
        /// Create a new permutation [Iterator].
        fn new(sum: T, number: usize) -> Self {
            let mut first_iter = T::zero()..=sum;
            let first = first_iter.next().unwrap();

            Self {
                sum,
                number,
                first_iter,
                first,
                rest_iter: Self::new_rest_iter(sum - first, number - 1),
            }
        }

        /// Creates a new iterator for the rest of the elements, which may be
        /// recursive or a single-permutation iterator if only a single element
        /// is needed.
        fn new_rest_iter(sum: T, number: usize) -> Box<dyn Iterator<Item = Vec<T>>> {
            if number > 1 {
                Box::new(Self::new(sum, number))
            } else {
                Box::new([vec![sum]].into_iter())
            }
        }
    }
    impl<T> Iterator for SumPermutations<T>
    where
        T: Copy + Step + num::Num + 'static,
    {
        type Item = Vec<T>;

        fn next(&mut self) -> Option<Self::Item> {
            match self.rest_iter.next() {
                Some(mut perm) => {
                    // There is another permutation for the rest, so just prepend the current first value
                    perm.insert(0, self.first);
                    Some(perm)
                }
                None => {
                    // There are no more permutations for the rest.
                    match self.first_iter.next() {
                        Some(first) => {
                            // We have more permutations for the next first value
                            self.first = first;
                            self.rest_iter = Self::new_rest_iter(self.sum - first, self.number - 1);
                            self.next()
                        }
                        None => {
                            // We are all done, nothing else
                            None
                        }
                    }
                }
            }
        }
    }

    /// Behavior specific to a particular problem part
    pub trait Part {
        /// Determines if a cookie with particular total ingredients is valid to consider for the part.
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
    preprocessor: Some(|input| Ok(Box::new(input.parse::<Problem>()?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Problem>()?
                .best_recipe::<PartOne>()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Problem>()?
                .best_recipe::<PartTwo>()
                .into())
        },
    ],
};
