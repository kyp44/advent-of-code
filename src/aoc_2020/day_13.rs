use itertools::Itertools;
use nom::{
    bytes::complete::{is_not, tag},
    character::complete::{digit1, multispace1, space0},
    combinator::map,
    multi::separated_list1,
    sequence::{separated_pair, tuple},
};
use num::integer::gcd;
use std::convert::TryInto;

use crate::aoc::{AocError, ParseResult, Parseable, Solution};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![1895, 840493039281088],
    "939
    7,13,x,x,59,x,31,19",
        vec![Some(295), Some(1068781)],
        "0
    67,7,59,61",
        vec![None, Some(754018)],
        "0
    67,x,7,59,61",
        vec![None, Some(779210)],
        "0
    67,7,x,59,61",
        vec![None, Some(1261476)],
        "0
    1789,37,47,1889",
        vec![None, Some(1202161486)]
    }
}

#[derive(Debug)]
struct Schedule {
    earliest_time: u64,
    bus_ids: Vec<Option<u64>>,
}

impl Parseable<'_> for Schedule {
    fn parser(input: &str) -> ParseResult<Self> {
        map(
            separated_pair(
                digit1,
                multispace1,
                separated_list1(tuple((space0, tag(","), space0)), is_not(", \t\n\r")),
            ),
            |(ts, vs): (&str, Vec<&str>)| Schedule {
                earliest_time: ts.parse().unwrap(),
                bus_ids: vs.into_iter().map(|s| s.parse().ok()).collect(),
            },
        )(input)
    }
}

impl Schedule {
    fn valid_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.bus_ids.iter().filter_map(|id| *id)
    }
}

/// Returns -d (mod m).
/// Note that is correct and differs from m - (d % m) when d = 0.
fn neg_modulo(d: &u64, m: &u64) -> u64 {
    let md: i64 = -TryInto::<i64>::try_into(*d).unwrap();
    let m: i64 = (*m).try_into().unwrap();
    (md.rem_euclid(m)).try_into().unwrap()
}

struct ModuloValues {
    v: u64,
    m: u64,
}

impl ModuloValues {
    fn new(a: &u64, m: &u64) -> ModuloValues {
        ModuloValues { v: *a % *m, m: *m }
    }
}

impl Iterator for ModuloValues {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let r = Some(self.v);
        self.v += self.m;
        r
    }
}

pub const SOLUTION: Solution = Solution {
    day: 13,
    name: "Shuttle Search",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let schedule = Schedule::from_str(input)?;

            // Process
            let time_until = |id: &u64| neg_modulo(&schedule.earliest_time, id);
            let bus_id = schedule
                .valid_ids()
                .min_by(|a, b| time_until(a).cmp(&time_until(b)))
                .unwrap();
            Ok(bus_id * time_until(&bus_id))
        },
        // Part b)
        |input| {
            // Generation
            let schedule = Schedule::from_str(input)?;

            // Process
            // This problem is effectively the Chinese Remainder Theorem to solve a system
            // of modulo congruences. These can be solved so long as the modulo factors
            // (in our case the set of bus IDs) are all pairwise co-prime. So first we check
            // that this is the case to guarantee that there will be a solution.
            for v in schedule.valid_ids().combinations(2) {
                if gcd(v[0], v[1]) > 1 {
                    return Err(AocError::Process(format!(
                        "Part b) may not be solveable because {} and {} are not co-prime",
                        v[0], v[1]
                    )));
                }
            }
            // First get an iterator of tuples of (a, m), where a is congruence (time
            // between timestamp and bus leaving) and m is the modulo value (bus ID)
            // for each bus and ordered in descending order by m, which results in
            // the fastest solution.
            let mut conditions = schedule
                .bus_ids
                .iter()
                .enumerate()
                .filter_map(|(i, ido)| -> Option<(u64, u64)> {
                    match ido {
                        Some(id) => Some((neg_modulo(&i.try_into().unwrap(), id), *id)),
                        None => None,
                    }
                })
                .sorted_by(|t1, t2| t1.1.cmp(&t2.1).reverse());
            // Now we use a sieve search as described at
            // https://en.wikipedia.org/wiki/Chinese_remainder_theorem#Search_by_sieving
            let (mut t, mut m) = conditions.next().unwrap();
            for (na, nm) in conditions {
                for x in ModuloValues::new(&t, &m) {
                    if (x % nm) == na {
                        // Found a solution that meets all conditions so far
                        t = x;
                        m *= nm;
                        break;
                    }
                }
            }
            Ok(t)
        },
    ],
};
