use std::{collections::HashSet, fmt};

use nom::{
    bytes::complete::tag, character::complete::alpha1, combinator::map, sequence::separated_pair,
};

use crate::aoc::{prelude::*, trim};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![],
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

struct Replacement<'a> {
    from: &'a str,
    to: &'a str,
}
impl<'a> Parseable<'a> for Replacement<'a> {
    fn parser(input: &'a str) -> NomParseResult<Self>
    where
        Self: Sized,
    {
        map(
            separated_pair(alpha1, trim(tag("=>")), alpha1),
            |(from, to)| Replacement { from, to },
        )(input)
    }
}
impl<'a> fmt::Debug for Replacement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} => {}", self.from, self.to)
    }
}

struct StrReplacement<'a> {
    precedent: &'a str,
    replaced: String,
}

#[derive(Debug)]
struct Machine<'a> {
    replacements: Vec<Replacement<'a>>,
    medicine: &'a str,
}
impl<'a> Machine<'a> {
    fn from_str(s: &'a str) -> AocResult<Self> {
        let secs = s.trim().sections(2)?;

        Ok(Machine {
            replacements: Replacement::gather(secs[0].lines())?,
            medicine: secs[1],
        })
    }

    fn replace_idx_iter<'b: 'a>(
        &'a self,
        input: &'b str,
        idx: usize,
    ) -> impl Iterator<Item = StrReplacement<'b>> + 'a {
        let pre = &input[..idx];
        let check = &input[idx..];

        self.replacements
            .iter()
            .filter(|r| check.starts_with(r.from))
            .map(move |r| StrReplacement {
                precedent: pre,
                replaced: format!("{}{}", pre, check.replacen(r.from, r.to, 1)),
            })
    }

    fn replace_iter<'b: 'a>(
        &'a self,
        input: &'b str,
    ) -> impl Iterator<Item = StrReplacement<'b>> + 'a {
        (0..input.len())
            .map(|idx| self.replace_idx_iter(input, idx))
            .flatten()
    }

    /*fn find_steps(&self, target: &str, input: &str) -> Option<u64> {
        println!("{}", input);
        if input == target {
            return Some(0);
        }

        for idx in 0..input.len() {
            let (pre, post) = input.split_at(idx);
            for rep in self.replacements.iter().filter(|r| post.starts_with(r.to)) {
                if let Some(s) = self.find_steps(
                    target,
                    &format!("{}{}", pre, post.replacen(rep.to, rep.from, 1)),
                ) {
                    return Some(s + 1);
                }
            }
        }

        None
    }*/

    fn find_steps(&self, target: &str, input: &str) -> Option<u64> {
        fn find_steps_rec<'a>(
            replacements: &[Replacement<'_>],
            bad_strs: &mut HashSet<String>,
            target: &str,
            input: String,
        ) -> Option<u64> {
            if input == target {
                return Some(0);
            } else if bad_strs.contains(&input) || input.find(target).is_some() {
                // An assumption here is that the target string is not a part
                // of any replacement to string, i.e. it cannot be further transformed.
                // Thus, if it is an any non-equal string, this branch can be abandoned.
                return None;
            }
            println!("{}", input);

            // Try replacements recursively
            for rep in replacements.iter() {
                for rs in input.individual_replacements(rep.to, rep.from) {
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

            /*println!("{:?}", machine);
            for s in machine.replace_iter(machine.medicine) {
                println!("{}", s.replaced);
            }*/

            // Process
            let set: HashSet<String> = machine
                .replace_iter(machine.medicine)
                .map(|sr| sr.replaced)
                .collect();
            Ok(Answer::Unsigned(set.len().try_into().unwrap()))
        },
        // Part b)
        |input| {
            // Generation
            let machine = Machine::from_str(input)?;

            // Process
            Ok(machine
                .find_steps("e", machine.medicine)
                .ok_or_else(|| AocError::Process("Solution not found!".into()))?
                .into())
        },
    ],
};
