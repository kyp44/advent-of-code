use aoc::prelude::*;
use std::{collections::HashSet, str::FromStr};

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2";
            answers = unsigned![13, 1];
        }
        example {
            input = "R 5
U 8
L 8
D 3
R 17
D 10
L 25
U 20";
            answers = &[None, Some(Unsigned(36))];
        }
        actual_answers = unsigned![5779, 2331];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use cgmath::{EuclideanSpace, Point2, Vector2};
    use nom::{
        branch::alt, bytes::complete::tag, character::complete::space1, combinator::map,
        sequence::separated_pair,
    };
    use std::iter::FusedIterator;

    /// A cardinal direction in which to move in 2D.
    #[derive(Debug, Clone, Copy)]
    enum Direction {
        /// Move left.
        Left,
        /// Move right.
        Right,
        /// Move up.
        Up,
        /// Move down.
        Down,
    }
    impl Parsable<'_> for Direction {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            alt((
                map(tag("L"), |_| Self::Left),
                map(tag("R"), |_| Self::Right),
                map(tag("U"), |_| Self::Up),
                map(tag("D"), |_| Self::Down),
            ))(input)
        }
    }
    impl Direction {
        /// Returns the 2D displacement vector corresponding with this direction.
        ///
        /// This would be added to a position to move one space in the direction.
        pub fn as_vector(&self) -> Vector2<isize> {
            match self {
                Direction::Left => -Vector2::unit_x(),
                Direction::Right => Vector2::unit_x(),
                Direction::Up => Vector2::unit_y(),
                Direction::Down => -Vector2::unit_y(),
            }
        }
    }

    /// A move that can be made by the head of a rope.
    #[derive(Debug, Clone)]
    pub struct Move {
        /// The direction in which to move.
        direction: Direction,
        /// The number of spaces to move in the direction.
        spaces: u8,
    }
    impl Parsable<'_> for Move {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(Direction::parser, space1, nom::character::complete::u8),
                |(direction, spaces)| Move { direction, spaces },
            )(input)
        }
    }

    /// The type of the coordinates of the rope knots.
    type Coords = Point2<isize>;

    /// Represents the current state of a rope wth some number of knots.
    ///
    /// Note that the rope is guaranteed to have at least two (a head and tail) knots.
    #[derive(Clone)]
    pub struct Rope {
        /// The current coordinates of every knot on the rope.
        ///
        /// The head is element 0, with tails going on from there.
        knots: Vec<Coords>,
    }
    impl Rope {
        /// Creates a new rope with a particular number of knots.
        ///
        /// All of the knots will start in the same space.
        pub fn new(num_knots: usize) -> AocResult<Self> {
            if num_knots < 2 {
                Err(AocError::Process("Must have at least two knots!".into()))
            } else {
                Ok(Self {
                    knots: vec![Coords::origin(); num_knots],
                })
            }
        }
    }
    impl Rope {
        /// Moves the head of the rope a single space in a particular `direction`.
        ///
        /// The tails knots will also move following the tail movement rules.
        fn move_head(&mut self, direction: Direction) {
            // Move the head
            self.knots[0] += direction.as_vector();

            // Move the tails one by one in order
            for tail_idx in 1..self.knots.len() {
                let head = self.knots[tail_idx - 1];
                let tail = self.knots.get_mut(tail_idx).unwrap();
                let delta = head - *tail;

                // If the head is too far way, we need to move the tail to catch up,
                // though we can only ever move one space in any direction.
                if delta.x.abs() > 1 || delta.y.abs() > 1 {
                    tail.x += delta.x.signum();
                    tail.y += delta.y.signum();
                }
            }
        }

        /// Retrieves the current coordinates of the last tail knot.
        fn tail(&self) -> Coords {
            *self.knots.last().unwrap()
        }
    }

    /// An ordered list of rope head moves.
    #[derive(Debug)]
    pub struct MoveSet {
        /// The list of moves.
        moves: Vec<Move>,
    }
    impl FromStr for MoveSet {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                moves: Move::gather(s.lines())?,
            })
        }
    }
    impl MoveSet {
        /// Creates an [`Executor`] for this move set using a [`Rope`] with the specified number of knots.
        pub fn execute(
            &self,
            num_knots: usize,
        ) -> AocResult<Executor<impl Iterator<Item = &Move> + FusedIterator>> {
            Ok(Executor {
                first_emitted: false,
                rope: Rope::new(num_knots)?,
                moves_iter: self.moves.iter().filter(|muv| muv.spaces > 0),
                current_move: None,
            })
        }
    }

    /// An [`Iterator`] for executing a [`MoveSet`], that emits the last tail knot coordinates
    /// after each singular move.
    ///
    /// The first element emitted is the starting coordinates of the last tail knot.
    /// This can only be created by calling [`MoveSet::execute`].
    pub struct Executor<I> {
        /// Whether the initial tail coordinates have already been emitted.
        first_emitted: bool,
        /// The current state of the rope.
        rope: Rope,
        /// The iterator over each [`Move`].
        moves_iter: I,
        /// The move that is currently executing, if any.
        current_move: Option<Move>,
    }
    impl<'a, I: Iterator<Item = &'a Move> + FusedIterator> Iterator for Executor<I> {
        type Item = Coords;

        fn next(&mut self) -> Option<Self::Item> {
            // Emit the initial state if we have not already done so
            if !self.first_emitted {
                self.first_emitted = true;
                return Some(self.rope.tail());
            }

            if self
                .current_move
                .as_ref()
                .map(|m| m.spaces == 0)
                .unwrap_or(true)
            {
                // We are done with the current move so get the next one if there is one
                match self.moves_iter.next() {
                    Some(m) => self.current_move = Some(m.clone()),
                    None => {
                        self.current_move = None;
                        return None;
                    }
                }
            }

            // Move the rope a single step
            let muv = self.current_move.as_mut().unwrap();
            self.rope.move_head(muv.direction);
            muv.spaces -= 1;

            Some(self.rope.tail())
        }
    }
    impl<'a, I: Iterator<Item = &'a Move> + FusedIterator> FusedIterator for Executor<I> {}
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 9,
    name: "Rope Bridge",
    preprocessor: Some(|input| Ok(Box::new(MoveSet::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            let tail_positions: HashSet<_> = input.expect_data::<MoveSet>()?.execute(2)?.collect();
            Ok(u64::try_from(tail_positions.len()).unwrap().into())
        },
        // Part two
        |input| {
            // Process
            let tail_positions: HashSet<_> = input.expect_data::<MoveSet>()?.execute(10)?.collect();

            Ok(u64::try_from(tail_positions.len()).unwrap().into())
        },
    ],
};
