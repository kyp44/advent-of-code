use std::{cmp::min, collections::HashMap};

use nom::{
    bytes::complete::{tag, take_until},
    combinator::map,
    sequence::tuple,
};

use crate::aoc::{prelude::*, trim};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(2655), Unsigned(1059)],
    "Comet can fly 14 km/s for 10 seconds, but then must rest for 127 seconds.
    Dancer can fly 16 km/s for 11 seconds, but then must rest for 162 seconds.",
    vec![2660u64, 1564].answer_vec()
    }
}

struct Reindeer<'a> {
    name: &'a str,
    fly_speed: u64,
    fly_time: u64,
    rest_time: u64,
}
impl<'a> Parseable<'a> for Reindeer<'a> {
    fn parser(input: &'a str) -> NomParseResult<Self> {
        map(
            tuple((
                take_until(" "),
                trim(tag("can fly")),
                trim(nom::character::complete::u64),
                trim(tag("km/s for")),
                trim(nom::character::complete::u64),
                trim(tag("seconds, but then must rest for")),
                trim(nom::character::complete::u64),
                trim(tag("seconds.")),
            )),
            |(name, _, fly_speed, _, fly_time, _, rest_time, _)| Reindeer {
                name,
                fly_speed,
                fly_time,
                rest_time,
            },
        )(input.trim())
    }
}
impl Reindeer<'_> {
    fn distance_at(&self, time: u64) -> u64 {
        let period: u64 = self.fly_time + self.rest_time;
        let n_periods = time / period;
        let partial = time % period;
        self.fly_speed * (n_periods * self.fly_time + min(self.fly_time, partial))
    }
}

struct Race<'a> {
    reindeer: Box<[Reindeer<'a>]>,
}
impl<'a> Race<'a> {
    fn from_str(s: &'a str) -> AocResult<Self> {
        Ok(Race {
            reindeer: Reindeer::gather(s.lines())?.into_boxed_slice(),
        })
    }

    fn winners_at(&self, time: u64) -> Vec<&Reindeer<'a>> {
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

    fn run_new_race(&self, time: u64) -> u64 {
        let mut scores: HashMap<&str, u64> = self.reindeer.iter().map(|r| (r.name, 0)).collect();
        for t in 1..=time {
            for r in self.winners_at(t) {
                *scores.get_mut(r.name).unwrap() += 1;
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

const END_TIME: u64 = 2503;

pub const SOLUTION: Solution = Solution {
    day: 14,
    name: "Reindeer Olympics",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let race = Race::from_str(input)?;

            // Process
            let ans = race.winners_at(END_TIME)[0].distance_at(END_TIME);
            //println!("{}", ans);
            Ok(ans.into())
        },
        // Part b)
        |input| {
            // Generation
            let race = Race::from_str(input)?;

            // Process
            let ans = race.run_new_race(END_TIME);
            //println!("{}", ans);
            Ok(ans.into())
        },
    ],
};
