use aoc::prelude::*;
use std::{collections::HashSet, str::FromStr};

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
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

/// Contains solution implementation items.
///
/// I dislike this problem because a solution in a reasonable amount of time
/// depends on the specific input of the problem. In particular there are special
/// characters that can be transformed into their "meanings" when turning all the
/// element names into single characters for ease of understanding.
///
/// For a discussion of the properties of the input that allow this see
/// [this reddit post](https://www.reddit.com/r/adventofcode/comments/3xflz8/day_19_solutions/).
mod solution {
    use super::*;
    use aoc::parse::trim;
    use derive_new::new;
    use nom::{
        bytes::complete::tag, character::complete::alpha1, combinator::map,
        sequence::separated_pair,
    };
    use std::fmt;

    /// Replacement of an element that can be parsed from text input.
    #[derive(new)]
    struct Replacement {
        /// Element to be replaced.
        from: String,
        /// Element with which to replace.
        to: String,
    }
    impl Parseable<'_> for Replacement {
        fn parser(input: &str) -> NomParseResult<&str, Self>
        where
            Self: Sized,
        {
            map(
                separated_pair(alpha1, trim(false, tag("=>")), alpha1),
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
        /// Creates a replacement from string slices.
        fn from_strs(from: &str, to: &str) -> Self {
            Self::new(from.to_string(), to.to_string())
        }
    }

    /// Machine to make a medicine that can be parsed from text input.
    #[derive(Debug)]
    pub struct Machine {
        /// Possible replacements.
        replacements: Vec<Replacement>,
        /// Medicine molecule that we wish to make.
        pub medicine: String,
    }
    impl FromStr for Machine {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
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
                Replacement::from_strs("Rn", "("),
                Replacement::from_strs("Y", ","),
                Replacement::from_strs("Ar", ")"),
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
    }
    impl Machine {
        /// Returns an [`Iterator`] over each individual molecule replacement in an input molecule.
        pub fn replace_iter<'a>(&'a self, input: &'a str) -> impl Iterator<Item = String> + 'a {
            self.replacements
                .iter()
                .flat_map(|r| input.individual_replacements(&r.from, &r.to))
        }

        /// Counts the number of replacement steps required to create a target molecule
        /// from a starting molecule.
        pub fn number_of_steps(&self, target: &str, input: &str) -> Option<u64> {
            /// This is a recursive sub-function of [`Machine::number_of_steps`].
            fn number_of_steps_rec(
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
                        if let Some(i) = number_of_steps_rec(replacements, bad_strs, target, rs) {
                            return Some(i + 1);
                        }
                    }
                }

                // This string cannot be turned into the target.
                bad_strs.insert(input);
                None
            }

            number_of_steps_rec(
                &self.replacements,
                &mut HashSet::new(),
                target,
                input.to_string(),
            )
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 19,
    name: "Medicine for Rudolph",
    preprocessor: Some(|input| Ok(Box::new(Machine::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            let machine = input.expect_data::<Machine>()?;
            let set: HashSet<String> = machine.replace_iter(&machine.medicine).collect();
            Ok(Answer::Unsigned(set.len().try_into().unwrap()))
        },
        // Part two
        |input| {
            // Process
            let machine = input.expect_data::<Machine>()?;
            Ok(machine
                .number_of_steps("e", &machine.medicine)
                .ok_or(AocError::NoSolution)?
                .into())
        },
    ],
};
