use aoc::prelude::*;
use std::{collections::HashSet, str::FromStr};

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "H => HO
H => OH
O => HH

HOH";
            answers = unsigned![4];
        }
        example {
            input = "H => HO
H => OH
O => HH

HOHOHO";
        answers = unsigned![7];
    }
        example {
            input = "e => H
e => O
H => HO
H => OH
O => HH

HOH";
            answers = &[None, Some(Unsigned(3))];
        }
        example {
            input = "e => H
e => O
H => HO
H => OH
O => HH

HOHOHO";
            answers = &[None, Some(Unsigned(6))];
        }
        actual_answers = unsigned![576, 207];
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
    use aoc::{
        parse::trim,
        tree_search::{LeastStepsAction, LeastStepsTreeNode},
    };
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
        fn from_strings(from: &str, to: &str) -> Self {
            Self::new(from.to_string(), to.to_string())
        }
    }

    /// A molecule, which is a node in the tree search.
    ///
    /// The tree search works backwards, from the molecule we are trying to make
    /// to the starting molecule.
    struct Molecule<'a> {
        /// The molecule making machine.
        machine: &'a Machine,
        /// The current molecule string.
        current: String,
        /// The target molecule, which is the starting molecule.
        target: &'static str,
    }
    impl fmt::Debug for Molecule<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.current)
        }
    }
    impl<'a> Molecule<'a> {
        /// Returns the initial molecule, which is the molecule we want to make, for a given
        /// `start_molecule` and molecule making `machine`.
        fn start(start_molecule: &'static str, machine: &'a Machine) -> Self {
            Molecule {
                machine,
                current: machine.medicine.to_string(),
                target: start_molecule,
            }
        }
    }
    impl PartialEq for Molecule<'_> {
        fn eq(&self, other: &Self) -> bool {
            self.current == other.current
        }
    }
    impl Eq for Molecule<'_> {}
    impl std::hash::Hash for Molecule<'_> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.current.hash(state);
        }
    }
    impl LeastStepsTreeNode for Molecule<'_> {
        const STOP_AT_FIRST: bool = true;

        fn recurse_action(&self) -> aoc::tree_search::LeastStepsAction<Self> {
            if self.current == self.target {
                return LeastStepsAction::StopSuccess;
            } else if self.current.contains(self.target) {
                // An assumption here is that the target string is not a part
                // of any replacement to string, i.e. it cannot be further transformed.
                // Thus, if it is in any non-equal string, this branch can be abandoned.
                return LeastStepsAction::StopFailure;
            }

            // All replacements in the current string
            LeastStepsAction::Continue(
                self.machine
                    .replacements
                    .iter()
                    .flat_map(|rep| self.current.individual_replacements(&rep.to, &rep.from))
                    .map(|rep| Self {
                        machine: self.machine,
                        current: rep,
                        target: self.target,
                    })
                    .collect(),
            )
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
            // and the special marker chemicals (see notes above) with their characters,
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
                Replacement::from_strings("Rn", "("),
                Replacement::from_strings("Y", ","),
                Replacement::from_strings("Ar", ")"),
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
        pub fn number_of_steps(&self, starting_molecule: &'static str) -> Option<u64> {
            Molecule::start(starting_molecule, self)
                .least_steps()
                .map(|steps| steps.try_into().unwrap())
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
                .number_of_steps("e")
                .ok_or(AocError::NoSolution)?
                .into())
        },
    ],
};
