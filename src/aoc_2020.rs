use super::aoc::{AocError, ParseResult, Parseable, Solution, YearSolutions};
use nom::{
    error::context,
    sequence::separated_pair,
    character::complete::digit1,
    bytes::complete::tag,
    bytes::complete::take,
    combinator::rest,
};
use lazy_static::lazy_static;
use std::str::FromStr;

lazy_static! {
    /// All of the solutions
    pub static ref YEAR_SOLUTIONS: YearSolutions = YearSolutions {
        year: 2020,
        solutions: vec![DAY_01,
                        DAY_02,
                        DAY_03,
        ],
    };
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::solution_test;

    solution_test! {
        day_01,
        DAY_01,
        "1721
979
366
299
675
1456",
        vec![514579, 241861950]
    }

    solution_test! {
        day_02,
        DAY_02,
        "1-3 a: abcde
1-3 b: cdefg
2-9 c: ccccccccc",
        vec![2, 1]
    }

    solution_test! {
        day_03,
        DAY_03,
        "..##.......
#...#...#..
.#....#..#.
..#.#...#.#
.#...##..#.
..#.##.....
.#.#.#....#
.#........#
#.##...#...
#...##....#
.#..#...#.#",
        vec![7, 336]
    }
}

const DAY_01: Solution = Solution {
    day: 1,
    name: "Report Repair",
    solver: |input| {
        type Expense = u32;
        impl Parseable for Expense {
            fn parse(input: &str) -> ParseResult<Self> {
                context(
                    "expense",
                    digit1,
                )(input.trim()).map(|(next, res)| {
                    (next, res.parse().unwrap())
                })
            }
        }

        // Generation
        let values = Expense::gather(input.lines())?;

        // Processing
        // Part a
        let mut answers: Vec<u32> = vec![];
        answers.push({
            let mut i = itertools::iproduct!(values.iter(), values.iter());
            loop {
                match i.next() {
                    Some((v1, v2)) => {
                        if v1 + v2 == 2020 {
                            break v1*v2;
                        }
                    },
                    None => {
                        return Err(AocError::Process("No two values add to 2020".to_string()));
                    }
                }
            }
        });
        // Part b
        answers.push({
            let mut i = itertools::iproduct!(values.iter(), values.iter(), values.iter());
            loop {
                match i.next() {
                    Some((v1, v2, v3)) => {
                        if v1 + v2 + v3 == 2020 {
                            break v1*v2*v3;
                        }
                    },
                    None => {
                        return Err(AocError::Process("No three values add to 2020".to_string()));
                    }
                }
            }
        });


        Ok(answers)
    },
};

const DAY_02: Solution = Solution {
    day: 2,
    name: "Password Philosophy",
    solver: |input| {
        #[derive(Debug)]
        struct PasswordPolicy {
            a: u32,
            b: u32,
            character: char,
        }

        impl Parseable for PasswordPolicy {
            fn parse(input: &str) -> ParseResult<Self> {
                context(
                    "password policy",
                    separated_pair(
                        separated_pair(digit1, tag("-"), digit1),
                        tag(" "),
                        take(1usize),
                    )
                )(input).map(|(next, res)| {
                    // Note that we can unwrap safely here because the range bounds should be digits
                    (next, PasswordPolicy{
                        a: res.0.0.parse().unwrap(),
                        b: res.0.1.parse().unwrap(),
                        character: res.1.chars().next().unwrap(),
                    })
                })
            }
        }

        #[derive(Debug)]
        struct Password {
            policy: PasswordPolicy,
            password: String,
        }
        impl Parseable for Password {
            fn parse(input: &str) -> ParseResult<Self> {
                context(
                    "password",
                    separated_pair(PasswordPolicy::parse, tag(": "), rest),
                )(input.trim()).map(|(next, res)| {
                    (next, Password{
                        policy: res.0,
                        password: res.1.to_string(),
                    })
                })
            }
        }
        impl Password {
            fn valid_part_a(&self) -> bool {
                let char_count = self.password.matches(self.policy.character).count() as u32;
                (self.policy.a..=self.policy.b).contains(&char_count)
            }

            fn valid_part_b(&self) -> bool {
                // Just going to naively assume that the string is long
                // enough to contain both characters
                macro_rules! check {
                    ($v:expr) => {
                        self.password.chars().nth(($v - 1) as usize).unwrap() == self.policy.character;
                    };
                }
                let a = check!(self.policy.a);
                let b = check!(self.policy.b);
                (a || b) && !(a && b)
            }
        }

        // Generation
        let passwords = Password::gather(input.lines())?;

        // Processing
        let mut answers = vec![];
        macro_rules! add_filter_count {
            ($a:expr) => {
                answers.push(passwords.iter().filter($a).count() as u32)
            };
        }
        add_filter_count!(|p| p.valid_part_a());
        add_filter_count!(|p| p.valid_part_b());

        Ok(answers)
    }
};

const DAY_03: Solution = Solution {
    day: 3,
    name: "Toboggan Trajectory",
    solver: |input| {
        struct Map {
            width: usize,
            height: usize,
            data: Vec<Vec<bool>>,
        }
        impl Map {
            fn is_tree(&self, x: usize, y: usize) -> bool {
                self.data[y][x % self.width]
            }
        }
        impl FromStr for Map {
            type Err = AocError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let mut liter = s.lines();
                fn parse_row(line: &str) -> Vec<bool> {
                    line.trim().chars().map(|c| !(c == '.')).collect()
                }
                let first_row = parse_row(liter.next()
                                    .ok_or(AocError::InvalidInput("No lines".to_string()))?);
                let width = first_row.len();
                if width < 1 {
                    return Err(AocError::InvalidInput("First map line has no content!".to_string()));
                }
                let mut data = vec![first_row];
                
                for line in liter {
                    let row = parse_row(line);
                    if row.len() != width {
                        return Err(AocError::InvalidInput(
                            format!("Map row '{}' has a length different from {}", line, width)
                        ));
                    }
                    data.push(row);
                }
                Ok(Map {
                    width,
                    height: data.len(),
                    data,
                })
            }
        }

        struct MapDownhill<'a> {
            map: &'a Map,
            dx: usize,
            dy: usize,
            x: usize,
            y: usize,
        }
        impl MapDownhill<'_> {
            fn new(map: &'_ Map, dx: usize, dy: usize) -> MapDownhill{
                MapDownhill {
                    map, dx, dy,
                    x: 0, y: 0,
                }
            }
        }
        impl Iterator for MapDownhill<'_> {
            type Item = bool;

            fn next(&mut self) -> Option<Self::Item> {
                // If past the map vertically then we are done
                if self.y >= self.map.height {
                    return None;
                }

                // Get current position
                let tree = self.map.is_tree(self.x, self.y);

                // Ready the next position
                self.x += self.dx;
                self.y += self.dy;

                Some(tree)
            }
        }

        // Generation
        let map = Map::from_str(input)?;

        // Process
        let mut answers = vec![];
        let count_slope = |x, y| {
            MapDownhill::new(&map, x, y).filter(|t| *t).count() as u32
        };
        // Part a
        answers.push(count_slope(3, 1));
        // Part b
        const SLOPES: [(usize, usize); 5] = [(1, 1), (3, 1), (5, 1), (7, 1), (1, 2)];
        answers.push(
            SLOPES.iter().map(|(x,y)| count_slope(*x,  *y)).product()
        );
        
        Ok(answers)
    }
};
