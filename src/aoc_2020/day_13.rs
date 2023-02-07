use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(1895), Unsigned(840493039281088)],
    "939
    7,13,x,x,59,x,31,19",
        vec![295u64, 1068781].answer_vec(),
        "0
    67,7,59,61",
        vec![None, Some(Unsigned(754018))],
        "0
    67,x,7,59,61",
        vec![None, Some(Unsigned(779210))],
        "0
    67,7,x,59,61",
        vec![None, Some(Unsigned(1261476))],
        "0
    1789,37,47,1889",
        vec![None, Some(Unsigned(1202161486))]
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use derive_new::new;
    use itertools::Itertools;
    use nom::{
        bytes::complete::{is_not, tag},
        character::complete::{multispace1, space0},
        combinator::map,
        multi::separated_list1,
        sequence::{separated_pair, tuple},
    };
    use num::integer::gcd;
    use std::convert::TryInto;

    /// The earliest bus we can take.
    #[derive(new)]
    pub struct EarliestBus {
        /// Earliest bus ID.
        pub bus_id: u64,
        /// Time we have to wait for this bus (minutes).
        pub wait_time: u64,
    }

    /// Bus schedule, which can be parsed from text input.
    #[derive(Debug)]
    pub struct Schedule {
        /// The earliest time we can depart.
        earliest_time: u64,
        /// List of bus IDs (`Some`), including buses that are not running (`None`).
        bus_ids: Vec<Option<u64>>,
    }
    impl Parseable<'_> for Schedule {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(
                    nom::character::complete::u64,
                    multispace1,
                    separated_list1(tuple((space0, tag(","), space0)), is_not(", \t\n\r")),
                ),
                |(earliest_time, vs): (u64, Vec<&str>)| Schedule {
                    earliest_time,
                    bus_ids: vs.into_iter().map(|s| s.parse().ok()).collect(),
                },
            )(input)
        }
    }
    impl Schedule {
        /// Returns an [`Iterator`] of all bus IDs, ignoring those that are not running.
        fn valid_ids(&self) -> impl Iterator<Item = u64> + '_ {
            self.bus_ids.iter().filter_map(|id| *id)
        }

        /// Determines the earlier bus that we can take.
        pub fn earliest_bus(&self) -> EarliestBus {
            let time_until = |id: u64| neg_modulo(self.earliest_time, id);
            let bus_id = self
                .valid_ids()
                .min_by(|a, b| time_until(*a).cmp(&time_until(*b)))
                .unwrap();

            EarliestBus::new(bus_id, time_until(bus_id))
        }

        /// Determines the earliest time at which buses depart in consecutive minutes, with gaps
        /// for non-running buses.
        pub fn earliest_consecutive_departures_time(&self) -> AocResult<u64> {
            // This problem is effectively the Chinese Remainder Theorem to solve a system
            // of modulo congruences. These can be solved so long as the modulo factors
            // (in our case the set of bus IDs) are all pairwise co-prime. So first we check
            // that this is the case to guarantee that there will be a solution.
            for v in self.valid_ids().combinations(2) {
                if gcd(v[0], v[1]) > 1 {
                    return Err(AocError::Process(
                        format!(
                            "Part two may not be solvable because {} and {} are not co-prime",
                            v[0], v[1]
                        )
                        .into(),
                    ));
                }
            }
            // First get an iterator of tuples of (a, m), where a is congruence (time
            // between timestamp and bus leaving) and m is the modulo value (bus ID)
            // for each bus and ordered in descending order by m, which results in
            // the fastest solution.
            let mut conditions = self
                .bus_ids
                .iter()
                .enumerate()
                .filter_map(|(i, ido)| -> Option<(u64, u64)> {
                    ido.map(|id| (neg_modulo(i.try_into().unwrap(), id), id))
                })
                .sorted_by(|t1, t2| t1.1.cmp(&t2.1).reverse());

            // Now we use a sieve search as described at
            // https://en.wikipedia.org/wiki/Chinese_remainder_theorem#Search_by_sieving
            let (mut t, mut m) = conditions.next().unwrap();
            for (na, nm) in conditions {
                for x in ModuloValues::new(t, m) {
                    if (x % nm) == na {
                        // Found a solution that meets all conditions so far
                        t = x;
                        m *= nm;
                        break;
                    }
                }
            }

            Ok(t)
        }
    }

    /// Returns `-d` modulo `m`.
    ///
    /// Note that is correct and differs from `m - (d % m)` when `d == 0`.
    fn neg_modulo(d: u64, m: u64) -> u64 {
        let md: i64 = -TryInto::<i64>::try_into(d).unwrap();
        let m: i64 = m.try_into().unwrap();
        (md.rem_euclid(m)).try_into().unwrap()
    }

    /// Endless [`Iterator`] over successive numbers that are all the same modulo some other number.
    struct ModuloValues {
        /// The next value in the sequence.
        current: u64,
        /// Modulo number.
        modulo: u64,
    }
    impl ModuloValues {
        /// Create a new [`Iterator`], starting at the lowest positive number and
        /// going through all numbers congruent to `a` modulo `modulo`.
        fn new(a: u64, modulo: u64) -> ModuloValues {
            ModuloValues {
                current: a % modulo,
                modulo,
            }
        }
    }
    impl Iterator for ModuloValues {
        type Item = u64;

        fn next(&mut self) -> Option<Self::Item> {
            let r = Some(self.current);
            self.current += self.modulo;
            r
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 13,
    name: "Shuttle Search",
    preprocessor: Some(|input| Ok(Box::new(Schedule::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            let earliest = input.expect_data::<Schedule>()?.earliest_bus();
            Ok((earliest.bus_id * earliest.wait_time).into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Schedule>()?
                .earliest_consecutive_departures_time()?
                .into())
        },
    ],
};
