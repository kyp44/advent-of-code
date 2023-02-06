use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(2655), Unsigned(1059)],
    "Comet can fly 14 km/s for 10 seconds, but then must rest for 127 seconds.
    Dancer can fly 16 km/s for 11 seconds, but then must rest for 162 seconds.",
    vec![2660u64, 1564].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use nom::{
        bytes::complete::{tag, take_until},
        combinator::map,
        sequence::tuple,
    };
    use std::{cmp::min, collections::HashMap};

    /// One of Santa's reindeer with its racing stats that can be parsed from text input.
    pub struct Reindeer {
        /// Name of the reindeer.
        name: String,
        /// Flying speed in km/s.
        fly_speed: u64,
        /// Time the reindeer can fly before resting in seconds.
        fly_time: u64,
        /// Time for which the reindeer must rest after flying before flying again in seconds.
        rest_time: u64,
    }
    impl Parseable<'_> for Reindeer {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                tuple((
                    take_until::<_, &str, _>(" "),
                    trim(false, tag("can fly")),
                    trim(false, nom::character::complete::u64),
                    trim(false, tag("km/s for")),
                    trim(false, nom::character::complete::u64),
                    trim(false, tag("seconds, but then must rest for")),
                    trim(false, nom::character::complete::u64),
                    trim(false, tag("seconds.")),
                )),
                |(name, _, fly_speed, _, fly_time, _, rest_time, _)| Reindeer {
                    name: name.to_string(),
                    fly_speed,
                    fly_time,
                    rest_time,
                },
            )(input.trim())
        }
    }
    impl Reindeer {
        /// Calculates the distance the reindeer has traveled after some time in seconds.
        pub fn distance_at(&self, time: u64) -> u64 {
            let period: u64 = self.fly_time + self.rest_time;
            let n_periods = time / period;
            let partial = time % period;
            self.fly_speed * (n_periods * self.fly_time + min(self.fly_time, partial))
        }
    }

    /// Overall race that can be parsed from text input.
    pub struct Race {
        /// Reindeer that are in the race.
        reindeer: Box<[Reindeer]>,
    }
    impl Race {
        /// Parse the race from text input.
        pub fn from_str(s: &str) -> AocResult<Self> {
            Ok(Race {
                reindeer: Reindeer::gather(s.lines())?.into_boxed_slice(),
            })
        }

        /// Determines the potentially multiple winners at a time in seconds.
        ///
        /// That is, which reindeer have traveled the furthest distance.
        pub fn winners_at(&self, time: u64) -> Vec<&Reindeer> {
            let dist = self
                .reindeer
                .iter()
                .map(|r| r.distance_at(time))
                .max()
                .unwrap();
            self.reindeer
                .iter()
                .filter(|r| r.distance_at(time) == dist)
                .collect()
        }

        /// Runs a race with the scoring used in part two.
        pub fn run_new_race(&self, time: u64) -> u64 {
            let mut scores: HashMap<&str, u64> =
                self.reindeer.iter().map(|r| (r.name.as_ref(), 0)).collect();
            for t in 1..=time {
                for r in self.winners_at(t) {
                    *scores.get_mut(&r.name.as_ref()).unwrap() += 1;
                }

                /*println!(
                    "{} {}",
                    t,
                    self.reindeer
                        .iter()
                        .map(|r| format!(
                            "{}: {} {}",
                            r.name,
                            r.distance_at(t),
                            scores.get(r.name).unwrap()
                        ))
                        .join(", ")
                );*/
            }

            *scores.values().max().unwrap()
        }
    }

    /// Time at which we are interested in the race results.
    pub const END_TIME: u64 = 2503;
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 14,
    name: "Reindeer Olympics",
    preprocessor: Some(|input| Ok(Box::new(Race::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            let ans = input.expect_data::<Race>()?.winners_at(END_TIME)[0].distance_at(END_TIME);
            //println!("{}", ans);
            Ok(ans.into())
        },
        // Part two
        |input| {
            // Process
            let ans = input.expect_data::<Race>()?.run_new_race(END_TIME);
            //println!("{}", ans);
            Ok(ans.into())
        },
    ],
};
