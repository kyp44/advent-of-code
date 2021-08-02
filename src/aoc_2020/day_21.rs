use crate::aoc::prelude::*;
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
    convert::TryInto,
    rc::Rc,
    str::FromStr,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Number;

    solution_test! {
    vec![Number(2287)],
    "mxmxvkd kfcds sqjhc nhms (contains dairy, fish)
trh fvjkl sbzzf mxmxvkd (contains dairy)
sqjhc fvjkl (contains soy)
sqjhc mxmxvkd sbzzf (contains fish)",
    vec![Some(Number(5)), Some(Answer::String("mxmxvkd,sqjhc,fvjkl".to_string()))]
    }
}

#[derive(Debug)]
struct Food {
    ingredients: HashSet<Rc<String>>,
    allergens: HashSet<Rc<String>>,
}
impl Parseable<'_> for Food {
    fn parser(input: &str) -> crate::aoc::NomParseResult<Self> {
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

type ProblemStr = Rc<String>;
struct PartialSolution {
    possibilities: HashMap<ProblemStr, HashSet<ProblemStr>>,
    safe_ingredients: HashSet<ProblemStr>,
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
    fn partial_solve(&self) -> PartialSolution {
        // First setup up a map of each allergen to possible ingredients,
        // initializing to every ingredient.
        let mut possibilities: HashMap<ProblemStr, HashSet<ProblemStr>> = self
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
            .map(|ings| ings.iter())
            .flatten()
            .cloned()
            .collect();
        let safe_ingredients = self
            .ingredients
            .difference(&unsafe_ingredients)
            .cloned()
            .collect();

        PartialSolution {
            possibilities,
            safe_ingredients,
        }
    }

    fn finish_solve(&self, partial_solution: PartialSolution) -> HashMap<ProblemStr, ProblemStr> {
        let mut final_ingredients = HashMap::new();

        // Lastly, we repeatly pare down the possibilities when allergens
        // have only a single possible ingredient
        for allergen in partial_solution.possibilities.keys() {
            let ingredients = &mut partial_solution.possibilities[allergen];

            if ingredients.len() == 1 {
                final_ingredients.insert(allergen.clone(), ingredients.drain().next().unwrap());
            }
        }

        final_ingredients
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
            let partial_solution = problem.partial_solve();

            println!("TODO: {:?}", partial_solution.possibilities);
            println!("TODO: {:?}", partial_solution.safe_ingredients);

            // Finally, count the number of times these appear in the ingedients list.
            /*Ok(Answer::Number(
                    problem
                        .foods
                        .iter()
                        .map(|food| {
                            partial_solution
                                .safe_ingredients
                                .intersection(&Problem::slices(&food.ingredients).collect())
                                .count()
                        })
                        .sum::<usize>()
                        .try_into()
                        .unwrap(),
            ))*/
            Ok(0.into())
        },
        // Part b)
        |input| {
            // Generation
            let problem = Problem::from_str(input)?;

            // Process
            //let full_solution = problem.finish_solve(problem.partial_solve());
            Ok(Answer::String("giggles".to_string()))
        },
    ],
};
