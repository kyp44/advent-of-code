use aoc::prelude::*;
use itertools::Itertools;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "mxmxvkd kfcds sqjhc nhms (contains dairy, fish)
trh fvjkl sbzzf mxmxvkd (contains dairy)
sqjhc fvjkl (contains soy)
sqjhc mxmxvkd sbzzf (contains fish)";
            answers = vec![Some(Unsigned(5)), Some(Answer::String("mxmxvkd,sqjhc,fvjkl".to_string()))];
        }
        actual_answers = vec![Unsigned(2287), Answer::String("fntg,gtqfrp,xlvrggj,rlsr,xpbxbv,jtjtrd,fvjkp,zhszc".to_string())];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use itertools::iproduct;
    use nom::{
        bytes::complete::tag,
        character::complete::{alphanumeric1, space1},
        combinator::{map, opt},
        multi::separated_list1,
        sequence::{delimited, pair},
    };
    use std::{
        collections::{HashMap, HashSet},
        rc::Rc,
    };

    /// A food, which can be parsed from text input.
    #[derive(Debug)]
    pub struct Food {
        /// List of ingredients that the food contains.
        ingredients: StrSet,
        /// List of allergens contained in any of the ingredients for this food.
        allergens: StrSet,
    }
    impl Parseable<'_> for Food {
        fn parser(input: &str) -> NomParseResult<&str, Self>
        where
            Self: Sized,
        {
            map(
                pair(
                    separated_list1(space1, alphanumeric1),
                    opt(delimited(
                        pair(space1, tag("(contains ")),
                        separated_list1(pair(tag(","), space1), alphanumeric1),
                        tag(")"),
                    )),
                ),
                |(iv, ao): (Vec<&str>, Option<Vec<&str>>)| Food {
                    ingredients: iv.into_iter().map(|s| Rc::new(s.to_string())).collect(),
                    allergens: match ao {
                        Some(av) => av.into_iter().map(|s| Rc::new(s.to_string())).collect(),
                        None => HashSet::new(),
                    },
                },
            )(input)
        }
    }
    impl Food {
        /// Gets the set of ingredients.
        pub fn ingredients(&self) -> &StrSet {
            &self.ingredients
        }
    }

    /// Shared strings used in the problem solution.
    type ProblemStr = Rc<String>;

    /// Set of problem strings.
    type StrSet = HashSet<ProblemStr>;

    /// A partial solution to the problem (all that is needed for part one).
    pub struct PartialSolution {
        /// The problem definition that generated this partial solution.
        problem: Problem,
        /// Map of allergens to the set ingredients that could possibly contain them.
        possibilities: HashMap<ProblemStr, StrSet>,
        /// Ingredients that definitely do not contain any allergens.
        safe_ingredients: StrSet,
    }
    impl PartialSolution {
        /// Returns an [`Iterator`] over all of the foods in the problem.
        pub fn foods(&self) -> impl Iterator<Item = &Food> {
            self.problem.foods.iter()
        }

        /// Sets of ingredients that definitely do not contain any allergens.
        pub fn safe_ingredients(&self) -> &StrSet {
            &self.safe_ingredients
        }

        /// Completes the full problem solution by determining in which ingredient each allergen is contained
        /// (needed for part two).
        pub fn finish_solve(&self) -> AocResult<FullSolution> {
            let mut final_ingredients = HashMap::new();
            let mut possibilities = self.possibilities.clone();

            // Lastly, we repeatedly pare down the possibilities when allergens
            // have only a single possible ingredient
            loop {
                let mut changed = false;
                for allergen in self.problem.allergens.iter() {
                    let ingredients = possibilities.get_mut(allergen).unwrap();

                    // If there is only one then set it in our final map
                    if ingredients.len() == 1 {
                        final_ingredients
                            .insert(allergen.clone(), ingredients.drain().next().unwrap());
                        changed = true;
                    }

                    // Now remove all known ingredients
                    for ingredient in final_ingredients.values() {
                        if ingredients.remove(ingredient) {
                            changed = true;
                        }
                    }
                }

                // If there was no change on this iteration then we are done
                if !changed {
                    break;
                }
            }

            if final_ingredients.len() == self.possibilities.len() {
                Ok(final_ingredients)
            } else {
                Err(AocError::Process("No final solution found".into()))
            }
        }
    }

    /// Full solution that maps each allergen to the ingredient that contains it.
    type FullSolution = HashMap<ProblemStr, ProblemStr>;

    /// Problem definition, which can be parsed from text input.
    ///
    /// NOTE: I tried to originally have the [`HashSet`]s own the [`String`]s
    /// and then have the [`Food`] vectors reference them, but self
    /// referential structs are not possible in Rust without [`std::pin::Pin`]
    /// and unsafe code.
    pub struct Problem {
        /// List of all ingredients.
        ingredients: StrSet,
        /// List of all allergens.
        allergens: StrSet,
        /// List of all foods.
        foods: Vec<Food>,
    }
    impl FromStr for Problem {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let foods = Food::gather(s.lines())?;

            /// This is an internal function for [`Problem::from_str`].
            ///
            /// Creates a set of a particular element of a [`Food`] based on
            /// a mapping function for all foods.
            fn convert(foods: &[Food], f: fn(&Food) -> &StrSet) -> StrSet {
                foods
                    .iter()
                    .flat_map(|food| f(food).iter().cloned())
                    .collect()
            }

            // Now gather up all the ingredients and allergens into a set
            Ok(Problem {
                ingredients: convert(&foods, |food| &food.ingredients),
                allergens: convert(&foods, |food| &food.allergens),
                foods,
            })
        }
    }
    impl Problem {
        /// Solves the problem partially, consuming the problem definition itself.
        pub fn partial_solve(self) -> PartialSolution {
            // First setup up a map of each allergen to possible ingredients,
            // initializing to every ingredient.
            let mut possibilities: HashMap<ProblemStr, StrSet> = self
                .allergens
                .iter()
                .map(|al| (al.clone(), self.ingredients.clone()))
                .collect();

            // Now remove ingredients that cannot contain the allergen based on
            // the food list.
            for (allergen, food) in iproduct!(self.allergens.iter(), self.foods.iter()) {
                if food.allergens.contains(allergen) {
                    possibilities.insert(
                        allergen.clone(),
                        possibilities[allergen]
                            .intersection(&food.ingredients)
                            .cloned()
                            .collect(),
                    );
                }
            }

            // Now determine those ingredients that do not appear in the
            // possibilities for any allergen.
            let unsafe_ingredients = possibilities
                .values()
                .flat_map(|ings| ings.iter())
                .cloned()
                .collect();
            let safe_ingredients = self
                .ingredients
                .difference(&unsafe_ingredients)
                .cloned()
                .collect();

            PartialSolution {
                problem: self,
                possibilities,
                safe_ingredients,
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 21,
    name: "Allergen Assessment",
    preprocessor: Some(|input| Ok(Box::new(input.parse::<Problem>()?.partial_solve()).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            let partial_solution = input.expect_data::<PartialSolution>()?;

            // Count the number of times these appear in the ingredients list.
            Ok(Answer::Unsigned(
                partial_solution
                    .foods()
                    .map(|food| {
                        partial_solution
                            .safe_ingredients()
                            .intersection(food.ingredients())
                            .count()
                    })
                    .sum::<usize>()
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<PartialSolution>()?
                .finish_solve()?
                .iter()
                .sorted_by(|(a1, _), (a2, _)| a1.cmp(a2))
                .map(|(_, i)| i)
                .join(",")
                .into())
        },
    ],
};
