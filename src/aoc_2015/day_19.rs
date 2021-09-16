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
        write!(f, "{} -> {}", self.from, self.to)
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
                .find_steps("e")
                .ok_or_else(|| AocError::Process("Solution not found!".into()))?
                .into())
        },
    ],
};
