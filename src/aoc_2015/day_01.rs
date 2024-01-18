use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;
    use Answer::Signed;

    solution_tests! {
        example {
            input = "(())";
            answers = signed![0];
        }
        example {
            input = "()()";
            answers = signed![0];
        }
        example {
            input = "(((";
            answers = signed![3];
        }
        example {
            input = "(()(()(";
            answers = signed![3];
        }
        example {
            input = "))(((((";
            answers = signed![3];
        }
        example {
            input = "())";
            answers = signed![-1];
        }
        example {
            input = "))(";
            answers = signed![-1];
        }
        example {
            input = ")))";
            answers = signed![-3];
        }
        example {
            input = ")())())";
            answers = signed![-3];
        }
        example {
            input = ")";
            answers = &[None, Some(Signed(1))];
        }
        example {
            input = "()())";
            answers = &[None, Some(Signed(5))];
        }
        actual_answers = signed![74, 1795];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use nom::{character::complete::one_of, combinator::map, multi::many1};

    /// A direction in which Santa can go.
    enum Direction {
        /// Up a floor.
        Up,
        /// Down a floor.
        Down,
    }
    impl Parseable<'_> for Direction {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(one_of("()"), |c| match c {
                '(' => Direction::Up,
                ')' => Direction::Down,
                _ => panic!(),
            })(input)
        }
    }
    impl Direction {
        /// Changes in floor number if Santa goes in this direction.
        fn floor_change(&self) -> i64 {
            match self {
                Direction::Up => 1,
                Direction::Down => -1,
            }
        }
    }

    /// A step by step list of directions that can be parsed from text input.
    pub struct Directions {
        /// The list of directions.
        directions: Box<[Direction]>,
    }
    impl Parseable<'_> for Directions {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(many1(Direction::parser), |v| Directions {
                directions: v.into_boxed_slice(),
            })(input)
        }
    }
    impl Directions {
        /// Returns an [`Iterator`] of floor numbers when the directions are followed,
        /// starting at floor 0.
        pub fn floors(&self) -> impl Iterator<Item = i64> + '_ {
            self.directions.iter().scan(0i64, |a, d| {
                *a += d.floor_change();
                Some(*a)
            })
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 1,
    name: "Not Quite Lisp",
    preprocessor: Some(|input| Ok(Box::new(Directions::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Directions>()?
                .floors()
                .last()
                .unwrap()
                .into())
        },
        // Part two
        |input| {
            // Process
            let pos = input
                .expect_data::<Directions>()?
                .floors()
                .position(|f| f < 0)
                .ok_or_else(|| AocError::Process("Santa never goes into the basement".into()))?
                + 1;
            Ok(Answer::Signed(pos.try_into().unwrap()))
        },
    ],
};
