use std::str::FromStr;

use nom::combinator::map;

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(380612)],
    "3,4,3,1,2",
    vec![5934u64, 26984457539].answer_vec()
    }
}

struct Fish {
    timer: u8,
}
impl Parseable<'_> for Fish {
    fn parser(input: &str) -> NomParseResult<Self> {
        map(nom::character::complete::u8, |timer| Fish { timer })(input)
    }
}
impl Fish {
    fn next_day(&mut self) -> Option<Fish> {
        if self.timer == 0 {
            self.timer = 6;
            Some(Fish { timer: 8 })
        } else {
            self.timer -= 1;
            None
        }
    }
}

struct Simulation {
    fish: Vec<Fish>,
}
impl FromStr for Simulation {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Simulation {
            fish: Fish::from_csv(s)?,
        })
    }
}
impl Iterator for Simulation {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let next = Some(self.fish.len());
        let new_fish: Vec<Fish> = self.fish.iter_mut().filter_map(|f| f.next_day()).collect();
        self.fish.extend(new_fish);
        next
    }
}

pub const SOLUTION: Solution = Solution {
    day: 6,
    name: "Lanternfish",
    solvers: &[
        // Part a)
        |input| {
            // Generation and process
            Ok(u64::try_from(Simulation::from_str(input)?.nth(80).unwrap())
                .unwrap()
                .into())
        },
        // Part b)
        |input| {
            // Generation and process
            Ok(
                u64::try_from(Simulation::from_str(input)?.nth(256).unwrap())
                    .unwrap()
                    .into(),
            )
        },
    ],
};
