/// I dislike this problem beacuse a solution in a reasonable amount of time
/// depends on the specific input of the problem. In particular there are special
/// Characters that can be transformed into their "meanings" when turning all the
/// element names into single characters for ease of understanding.
///
/// For a disucssion of the properties of the input that allow this see:
/// https://www.reddit.com/r/adventofcode/comments/3xflz8/day_19_solutions/
use std::{collections::HashSet, fmt};

use nom::{
    bytes::complete::tag, character::complete::alpha1, combinator::map, sequence::separated_pair,
};

use crate::aoc::{parse::trim, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(576), Unsigned(207)],
    "H => HO
H => OH
O => HH

HOH",
    vec![4u64].answer_vec(),
        "H => HO
H => OH
O => HH

HOHOHO",
    vec![7u64].answer_vec(),
    "e => H
e => O
H => HO
H => OH
O => HH

HOH",
    vec![None, Some(Unsigned(3))],
        "e => H
e => O
H => HO
H => OH
O => HH

HOHOHO",
    vec![None, Some(Unsigned(6))]
    }
}

#[derive(new)]
struct Replacement {
    from: String,
    to: String,
}
impl Parseable<'_> for Replacement {
    fn parser(input: &str) -> NomParseResult<&str, Self>
    where
        Self: Sized,
    {
        map(
            separated_pair(alpha1, trim(tag("=>")), alpha1),
            |(f, t): (&str, &str)| Replacement {
                from: f.to_string(),
                to: t.to_string(),
            },
        )(input)
    }
}
impl fmt::Debug for Replacement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}' => '{}'", self.from, self.to)
    }
}
impl Replacement {
    fn from_str(from: &str, to: &str) -> Self {
        Self::new(from.to_string(), to.to_string())
    }
}

#[derive(Debug)]
struct Machine {
    replacements: Vec<Replacement>,
    medicine: String,
}
impl Machine {
    fn from_str(s: &str) -> AocResult<Self> {
        let secs = s.trim().sections(2)?;
        let mut replacements = Replacement::gather(secs[0].lines())?;

        // For simplicity we replace all two-letter chemical names with a single
        // and the special demarker chemicals (see notes above) with their characters,
        // making it so that every symbol is only a single character.
        let mut symbols = 'A'..'z';
        // Gets a new, unused single-letter symbol.
        let mut new_symbol = || loop {
            match symbols.next() {
                Some(c) => {
                    let s = c.to_string();
                    if replacements.iter().all(|r| r.from != s) && s != "Y" {
                        break Some(s);
                    }
                }
                None => break None,
            }
        };

        // Add meta-replacements for two-letter elements and special elements
        let mut meta_replacements = vec![
            Replacement::from_str("Rn", "("),
            Replacement::from_str("Y", ","),
            Replacement::from_str("Ar", ")"),
        ];
        for symbol in replacements.iter().map(|r| &r.from) {
            if symbol.len() > 1 && meta_replacements.iter().all(|r| r.from != *symbol) {
                meta_replacements.push(Replacement::new(symbol.clone(), new_symbol().unwrap()));
            }
        }

        // Now perform meta-replacements in the replacements and medicine
        let mut medicine = secs[1].to_string();
        for meta_rep in meta_replacements {
            for rep in replacements.iter_mut() {
                rep.to = rep.to.replace(&meta_rep.from, &meta_rep.to);
                rep.from = rep.from.replace(&meta_rep.from, &meta_rep.to);
            }
            medicine = medicine.replace(&meta_rep.from, &meta_rep.to);
        }

        // Now make the search greedy
        replacements.sort_unstable_by_key(|r| r.to.len());
        replacements.reverse();

        Ok(Machine {
            replacements,
            medicine,
        })
    }

    fn replace_iter<'a>(&'a self, input: &'a str) -> impl Iterator<Item = String> + 'a {
        self.replacements
            .iter()
            .flat_map(|r| input.individual_replacements(&r.from, &r.to))
    }

    fn find_steps(&self, target: &str, input: &str) -> Option<u64> {
        fn find_steps_rec(
            replacements: &[Replacement],
            bad_strs: &mut HashSet<String>,
            target: &str,
            input: String,
        ) -> Option<u64> {
            if input == target {
                return Some(0);
            } else if bad_strs.contains(&input) || input.contains(target) {
                // An assumption here is that the target string is not a part
                // of any replacement to string, i.e. it cannot be further transformed.
                // Thus, if it is in any non-equal string, this branch can be abandoned.
                return None;
            }
            //println!("{}", input);

            // Try replacements recursively
            for rep in replacements.iter() {
                for rs in input.individual_replacements(&rep.to, &rep.from) {
                    if let Some(i) = find_steps_rec(replacements, bad_strs, target, rs) {
                        return Some(i + 1);
                    }
                }
            }

            // This string cannot be turned into the target.
            bad_strs.insert(input);
            None
        }

        find_steps_rec(
            &self.replacements,
            &mut HashSet::new(),
            target,
            input.to_string(),
        )
    }
}

pub const SOLUTION: Solution = Solution {
    day: 19,
    name: "Medicine for Rudolph",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let machine = Machine::from_str(input)?;

            //println!("{:?}", machine);
            /*for s in machine.replace_iter(&machine.medicine) {
                println!("{}", s);
            }*/

            // Process
            let set: HashSet<String> = machine.replace_iter(&machine.medicine).collect();
            Ok(Answer::Unsigned(set.len().try_into().unwrap()))
        },
        // Part b)
        |input| {
            // Generation
            let machine = Machine::from_str(input)?;
            //println!("{:?}", machine);

            // Process
            Ok(machine
                .find_steps("e", &machine.medicine)
                .ok_or_else(|| AocError::Process("Solution not found!".into()))?
                .into())
        },
    ],
};
