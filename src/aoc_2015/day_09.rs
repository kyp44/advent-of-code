use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "London to Dublin = 464
London to Belfast = 518
Dublin to Belfast = 141";
            answers = vec![605u64, 982].answer_vec();
        }
        actual_answers = vec![Unsigned(251), Unsigned(898)];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::separated;
    use itertools::Itertools;
    use nom::{
        bytes::complete::{tag, take_until},
        combinator::map,
        sequence::separated_pair,
    };
    use std::{
        collections::{HashMap, HashSet},
        hash,
    };

    /// A distance from one city to another that can be parsed from text input.
    #[derive(Debug)]
    struct Distance<'a> {
        /// Name of the origin city.
        origin: &'a str,
        /// Name of the destination city.
        destination: &'a str,
        /// Distance between the two cities.
        distance: u64,
    }
    impl<'a> Parseable<'a> for Distance<'a> {
        fn parser(input: &'a str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(
                    separated_pair(take_until(" "), separated(tag("to")), take_until(" ")),
                    separated(tag("=")),
                    nom::character::complete::u64,
                ),
                |((source, destination), distance)| Distance {
                    origin: source,
                    destination,
                    distance,
                },
            )(input.trim())
        }
    }

    /// Represents an abstract transit from between cities without regard to
    /// which is the source and which is the destination.
    #[derive(Eq)]
    struct Transit<'a>(&'a str, &'a str);
    impl<'a> PartialEq for Transit<'a> {
        /// Returns whether two transits are equal, which should not depend on the order.
        fn eq(&self, other: &Self) -> bool {
            (self.0 == other.0 && self.1 == other.1) || (self.0 == other.1 && self.1 == other.0)
        }
    }
    impl<'a> hash::Hash for Transit<'a> {
        /// Hashes a transit so that two transits in the opposite order produce the same hash.
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

    /// Problem definition, which can be parsed from text input.
    pub struct Problem<'a> {
        /// Set of all cities names.
        cities: HashSet<&'a str>,
        /// Map of transits to numeric distances.
        distances: HashMap<Transit<'a>, u64>,
    }
    impl<'a> Problem<'a> {
        /// Parses the problem from text input.
        pub fn from_str(input: &'a str) -> AocResult<Self> {
            let mut cities = HashSet::new();
            let mut distances = HashMap::new();
            let raw_distances = Distance::gather(input.lines())?;

            for dist in raw_distances {
                cities.insert(dist.origin);
                cities.insert(dist.destination);
                distances.insert(Transit(dist.origin, dist.destination), dist.distance);
            }

            Ok(Problem { cities, distances })
        }

        /// Calculates the distance along the route travel through a list of cities.
        fn route_distance(&self, route: &[&str]) -> u64 {
            route
                .windows(2)
                .map(|ws| self.distances.get(&Transit(ws[0], ws[1])).unwrap())
                .sum()
        }

        /// Returns an [`Iterator`] over the distances for all possible routes.
        fn routes_distances(&self) -> impl Iterator<Item = u64> + '_ {
            self.cities
                .iter()
                .copied()
                .permutations(self.cities.len())
                .map(move |r| self.route_distance(&r))
        }

        /// Determines the shortest distance among all possible routes.
        pub fn shortest_distance(&self) -> u64 {
            self.routes_distances().min().unwrap()
        }

        /// Determines the longest distance among all possible routes.
        pub fn longest_distance(&self) -> u64 {
            self.routes_distances().max().unwrap()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 9,
    name: "All in a Single Night",
    // NOTE: Problem keeps references to input, so we cannot use a pre-processor.
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let problem = Problem::from_str(input.expect_input()?)?;

            // Process
            Ok(problem.shortest_distance().into())
        },
        // Part two
        |input| {
            // Generation
            let problem = Problem::from_str(input.expect_input()?)?;

            // Process
            Ok(problem.longest_distance().into())
        },
    ],
};
