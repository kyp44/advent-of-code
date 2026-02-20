use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use Answer::Unsigned;
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = ">";
            answers = &[Some(Unsigned(2)), None];
        }
        example {
            input = "^v";
            answers = &[None, Some(Unsigned(3))];
        }
        example {
            input = "^>v<";
            answers = unsigned![4, 3];
        }
        example {
            input = "^v^v^v^v^v";
            answers = unsigned![2, 11];
        }
        actual_answers = unsigned![2565, 2639];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use euclid::{
        default::{Point2D, Vector2D},
        vec2,
    };
    use nom::{character::complete::one_of, combinator::map, multi::many1};
    use std::collections::HashSet;

    /// The type for the coordinates of a house.
    type Point = Point2D<i32>;

    /// The type for a displacement vector between houses.
    type Vector = Vector2D<i32>;

    /// A direction in which Santa can move.
    pub enum Direction {
        /// North (up).
        North,
        /// East (right).
        East,
        /// South (down).
        South,
        /// West (left).
        West,
    }
    impl Parsable for Direction {
            fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            use Direction::*;
            map(one_of("^>v<"), |s| match s {
                '^' => North,
                '>' => East,
                'v' => South,
                '<' => West,
                _ => panic!(),
            })
            .parse(input)
        }
    }
    impl Direction {
        /// Returns a direction vector to move one house in this direction.
        fn to_vector(&self) -> Vector {
            use Direction::*;
            match self {
                North => vec2(0, 1),
                East => vec2(1, 0),
                South => vec2(0, -1),
                West => vec2(-1, 0),
            }
        }
    }

    /// Behavior different for each part of the problem.
    pub trait Part {
        /// Returns a set of all house coordinates that Santa will visit given
        /// the list of directions to move.
        fn visited_houses(directions: &[Direction]) -> HashSet<Point>;
    }

    /// Behavior for part one.
    pub struct PartOne;
    impl Part for PartOne {
        fn visited_houses(directions: &[Direction]) -> HashSet<Point> {
            let mut vh: HashSet<Point> = directions
                .iter()
                .scan(Point::origin(), |a, d| {
                    *a += d.to_vector();
                    Some(*a)
                })
                .collect();
            vh.insert(Point::origin());
            vh
        }
    }

    /// Behavior for part two.
    pub struct PartTwo;
    impl Part for PartTwo {
        fn visited_houses(directions: &[Direction]) -> HashSet<Point> {
            let mut vh = HashSet::new();
            vh.insert(Point::origin());
            let mut santa = Point::origin();
            let mut robo = Point::origin();
            let mut santa_turn = true;
            for dir in directions {
                if santa_turn {
                    santa += dir.to_vector();
                    vh.insert(santa);
                } else {
                    robo += dir.to_vector();
                    vh.insert(robo);
                }
                santa_turn = !santa_turn;
            }
            vh
        }
    }

    /// A list of directions that can be parsed from text input.
    pub struct Directions {
        /// The list of directions.
        directions: Vec<Direction>,
    }
    impl Parsable for Directions {
            fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(many1(Direction::parser), |directions| Directions {
                directions,
            })
            .parse(input)
        }
    }
    impl Directions {
        /// Returns a set of all house coordinates that Santa will visit by
        /// following these directions.
        pub fn visited_houses<P: Part>(&self) -> HashSet<Point> {
            P::visited_houses(&self.directions)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 3,
    name: "Perfectly Spherical Houses in a Vacuum",
    preprocessor: Some(|input| Ok(Box::new(Directions::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<Directions>()?
                    .visited_houses::<PartOne>()
                    .len()
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<Directions>()?
                    .visited_houses::<PartTwo>()
                    .len()
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
