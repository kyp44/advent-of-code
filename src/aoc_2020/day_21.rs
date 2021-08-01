use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    rc::Rc,
    str::FromStr,
};

use itertools::iproduct;
use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, space1},
    combinator::{map, opt},
    multi::separated_list1,
    sequence::{delimited, pair},
};

use crate::aoc::{AocError, Parseable, Solution};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![2287],
    "mxmxvkd kfcds sqjhc nhms (contains dairy, fish)
trh fvjkl sbzzf mxmxvkd (contains dairy)
sqjhc fvjkl (contains soy)
sqjhc mxmxvkd sbzzf (contains fish)",
    vec![Some(5)]
    }
}

#[derive(Debug)]
struct Food {
    ingredients: HashSet<Rc<String>>,
    allergens: HashSet<Rc<String>>,
}
impl Parseable<'_> for Food {
    fn parser(input: &str) -> crate::aoc::ParseResult<Self> {
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

// NOTE: Tried to originally have the HashSets own the Strings
// and then have the Food vectors reference them, but apparently
// it is not possible to had a struct that contains references
// to data that it also owns. See
// https://stackoverflow.com/questions/30823880/struct-that-owns-some-data-and-a-reference-to-the-data
struct Problem {
    ingredients: HashSet<Rc<String>>,
    allergens: HashSet<Rc<String>>,
    foods: Vec<Food>,
}
impl FromStr for Problem {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let foods = Food::gather(s.lines())?;
        fn convert(foods: &[Food], f: fn(&Food) -> &HashSet<Rc<String>>) -> HashSet<Rc<String>> {
            foods
                .iter()
                .map(|food| f(food).iter().map(|i| i.clone()))
                .flatten()
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
    fn solve(&self) -> u64 {
        fn slices(set: &HashSet<Rc<String>>) -> impl Iterator<Item = &str> {
            set.iter().map(|rc| rc.as_str())
        }

        // First setup up a map of each allergen to possible ingredients,
        // initializing to every ingredient.
        let all_ingredients: HashSet<&str> = slices(&self.ingredients).collect();
        let mut possibilities: HashMap<&str, HashSet<&str>> = slices(&self.allergens)
            .map(|al| (al, all_ingredients.clone()))
            .collect();

        // Now remove ingredients that cannot contain the allergen based on
        // the food list.
        for (allergen, food) in iproduct!(slices(&self.allergens), self.foods.iter()) {
            let ingredients = slices(&food.ingredients).collect();
            if slices(&food.allergens).any(|a| a == allergen) {
                possibilities.insert(
                    allergen,
                    possibilities[allergen]
                        .intersection(&ingredients)
                        .copied()
                        .collect(),
                );
            }
        }

        // Now determine those ingredients that do not appear in the
        // possibilities for any allergen.
        let unsafe_ingredients: HashSet<&str> = possibilities
            .values()
            .map(|ings| ings.iter())
            .flatten()
            .copied()
            .collect();
        let safe_ingredients: HashSet<&str> = all_ingredients
            .difference(&unsafe_ingredients)
            .copied()
            .collect();

        // Finally, count the number of times these appear in the ingedients list.
        self.foods
            .iter()
            .map(|food| {
                safe_ingredients
                    .intersection(&slices(&food.ingredients).collect())
                    .count()
            })
            .sum::<usize>()
            .try_into()
            .unwrap()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 21,
    name: "Allergen Assessment",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let problem = Problem::from_str(input)?;

            // Process
            Ok(problem.solve())
        },
    ],
};
