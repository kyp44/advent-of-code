use std::{
    collections::{HashMap, HashSet},
    hash,
};

use itertools::Itertools;
use nom::{
    bytes::complete::{tag, take_until},
    combinator::map,
    sequence::separated_pair,
};

use crate::aoc::{prelude::*, separated};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(251), Unsigned(898)],
    "London to Dublin = 464
London to Belfast = 518
Dublin to Belfast = 141",
    vec![605u64, 982].answer_vec()
    }
}

#[derive(Debug)]
struct Distance<'a> {
    source: &'a str,
    destination: &'a str,
    distance: u64,
}

impl<'a> Parseable<'a> for Distance<'a> {
    fn parser(input: &'a str) -> NomParseResult<Self> {
        map(
            separated_pair(
                separated_pair(take_until(" "), separated(tag("to")), take_until(" ")),
                separated(tag("=")),
                nom::character::complete::u64,
            ),
            |((source, destination), distance)| Distance {
                source,
                destination,
                distance,
            },
        )(input.trim())
    }
}

#[derive(Eq)]
struct Transit<'a>(&'a str, &'a str);
impl<'a> PartialEq for Transit<'a> {
    /// Equality should not depend on the order.
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.0 && self.1 == other.1) || (self.0 == other.1 && self.1 == other.0)
    }
}
impl<'a> hash::Hash for Transit<'a> {
    /// Need a hash such that it is the same regardless of the order.
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        if self.0 <= self.1 {
            self.0.hash(state);
            self.1.hash(state);
        } else {
            self.1.hash(state);
            self.0.hash(state);
        }
    }
}

struct Problem<'a> {
    cities: HashSet<&'a str>,
    distances: HashMap<Transit<'a>, u64>,
}
impl<'a> Problem<'a> {
    fn from_str(input: &'a str) -> AocResult<Self> {
        let mut cities = HashSet::new();
        let mut distances = HashMap::new();
        let raw_distances = Distance::gather(input.lines())?;

        for dist in raw_distances {
            cities.insert(dist.source);
            cities.insert(dist.destination);
            distances.insert(Transit(dist.source, dist.destination), dist.distance);
        }

        Ok(Problem { cities, distances })
    }

    fn route_distance(&self, route: &[&str]) -> u64 {
        route
            .windows(2)
            .map(|ws| self.distances.get(&Transit(ws[0], ws[1])).unwrap())
            .sum()
    }

    fn routes_distances<'b>(&'b self) -> impl Iterator<Item = u64> + 'b {
        self.cities
            .iter()
            .copied()
            .permutations(self.cities.len())
            .map(move |r| self.route_distance(&r))
    }

    fn shortest_distance(&self) -> u64 {
        self.routes_distances().min().unwrap()
    }

    fn longest_distance(&self) -> u64 {
        self.routes_distances().max().unwrap()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 9,
    name: "All in a Single Night",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let problem = Problem::from_str(input)?;

            // Process
            Ok(problem.shortest_distance().into())
        },
        // Part b)
        |input| {
            // Generation
            let problem = Problem::from_str(input)?;

            // Process
            Ok(problem.longest_distance().into())
        },
    ],
};
