use std::collections::{HashMap, HashSet};

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

struct StrReplacement<'a> {
    precedent: &'a str,
    replaced: String,
}

#[derive(Debug)]
struct Machine<'a> {
    replacements: HashMap<&'a str, Vec<&'a str>>,
    medicine: &'a str,
}
impl<'a> Machine<'a> {
    fn from_str(s: &'a str) -> AocResult<Self> {
        let secs = s.trim().sections(2)?;

        let mut replacements: HashMap<_, Vec<&str>> = HashMap::new();

        for rep in Replacement::gather(secs[0].lines())? {
            match replacements.get_mut(rep.from) {
                Some(v) => v.push(rep.to),
                None => {
                    replacements.insert(rep.from, vec![rep.to]);
                }
            }
        }

        Ok(Machine {
            replacements,
            medicine: secs[1],
        })
    }

    fn replace_idx_iter<'b: 'a>(
        &'a self,
        input: &'b str,
        idx: usize,
    ) -> Option<impl Iterator<Item = StrReplacement<'b>> + 'a> {
        let pre = &input[..idx];
        let check = &input[idx..];
        for (from, vec) in self.replacements.iter() {
            if check.starts_with(from) {
                return Some(vec.iter().map(move |to| StrReplacement {
                    precedent: pre,
                    replaced: format!("{}{}", pre, check.replacen(from, to, 1)),
                }));
            }
        }
        None
    }

    fn replace_iter<'b: 'a>(
        &'a self,
        input: &'b str,
    ) -> impl Iterator<Item = StrReplacement<'b>> + 'a {
        (0..input.len())
            .filter_map(|idx| self.replace_idx_iter(input, idx))
            .flatten()
    }

    fn find_steps<'b: 'a>(&self, input: &'b str) -> Option<u64> {
        if input.len() >= self.medicine.len() {
            if input == self.medicine {
                return Some(0);
            }
            return None;
        }

        for s in self.replace_iter(input) {
            // If what we have replaced so far doesn't match, there's no
            // point in continuing down this dead end.
            if !self.medicine.starts_with(s.precedent) {
                return None;
            }
            //println!("Considering with prec '{}'", s.precedent,);
            if let Some(i) = self.find_steps(&s.replaced) {
                return Some(i + 1);
            }
        }

        None
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
                for s in machine.replace_iter() {
                    println!("{}", s);
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
                .find_steps("e")
                .ok_or_else(|| AocError::Process("Solution not found!".into()))?
                .into())
        },
    ],
};
