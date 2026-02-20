use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use Answer::Unsigned;
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "939
    7,13,x,x,59,x,31,19";
            answers = unsigned![295, 1068781];
        }
        example {
            input = "0
    67,7,59,61";
            answers = &[None, Some(Unsigned(754018))];
        }
        example {
            input = "0
    67,x,7,59,61";
            answers = &[None, Some(Unsigned(779210))];
        }
        example {
            input = "0
    67,7,x,59,61";
            answers = &[None, Some(Unsigned(1261476))];
        }
        example {
            input = "0
    1789,37,47,1889";
            answers = &[None, Some(Unsigned(1202161486))];
        }
        actual_answers = unsigned![1895, 840493039281088];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use bare_metal_modulo::{MNum, ModNum};
    use derive_new::new;
    use itertools::Itertools;
    use nom::{
        bytes::complete::{is_not, tag},
        character::complete::{multispace1, space0},
        combinator::map,
        multi::separated_list1,
        sequence::separated_pair,
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
        /// List of bus IDs (`Some`), including buses that are not running
        /// (`None`).
        bus_ids: Vec<Option<ModNum<u64>>>,
    }
    impl Parsable for Schedule {
            fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(
                separated_pair(
                    nom::character::complete::u64,
                    multispace1,
                    separated_list1((space0, tag(","), space0), is_not(", \t\n\r")),
                ),
                |(earliest_time, vs): (u64, Vec<&str>)| Schedule {
                    earliest_time,
                    bus_ids: vs
                        .into_iter()
                        .map(|s| s.parse().ok().map(|id| ModNum::new(0, id)))
                        .collect(),
                },
            )
            .parse(input)
        }
    }
    impl Schedule {
        /// Returns an [`Iterator`] of all bus IDs, ignoring those that are not
        /// running.
        fn valid_ids(&self) -> impl Iterator<Item = ModNum<u64>> + '_ {
            self.bus_ids.iter().filter_map(|id| *id)
        }

        /// Determines the earlier bus that we can take.
        pub fn earliest_bus(&self) -> EarliestBus {
            let time_until = |id| id - self.earliest_time;
            let bus_id = self.valid_ids().min_by_key(|id| time_until(*id)).unwrap();

            EarliestBus::new(bus_id.m(), time_until(bus_id).a())
        }

        /// Determines the earliest time at which buses depart in consecutive
        /// minutes, with gaps for non-running buses.
        pub fn earliest_consecutive_departures_time(&self) -> AocResult<u64> {
            // This problem is effectively the Chinese Remainder Theorem to solve a system
            // of modulo congruences.
            // These can be solved so long as the modulo factors (in our case the set of bus
            // IDs) are all pairwise co-prime. So first we check that this is the case to
            // guarantee that there will be a solution.
            for v in self.valid_ids().combinations(2) {
                if gcd(v[0].m(), v[1].m()) > 1 {
                    return Err(AocError::Process(
                        format!(
                            "Part two may not be solvable because {} and {} are not co-prime",
                            v[0], v[1]
                        )
                        .into(),
                    ));
                }
            }

            // The solution will be the earliest timestamp.
            // So first form an iterator of modulo numbers, to all of which the solution
            // must be congruent.
            let mod_iter = self.bus_ids.iter().enumerate().filter_map(|(i, id)| {
                id.map(|m| -ModNum::<i128>::new(i.try_into().unwrap(), m.m().into()))
            });

            // Solve the congruence system.
            // Note: This requires signed numeric types for some reason, which is annoying.
            Ok(ModNum::chinese_remainder_system(mod_iter)
                .unwrap()
                .a()
                .try_into()
                .unwrap())
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
