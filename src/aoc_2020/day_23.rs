use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "389125467";
            answers = &[Some(Unsigned(67384529)), None];
        }
        expensive_example {
            input = "389125467";
            answers = &[None, Some(Unsigned(149245887792))];
        }
        actual_answers = unsigned![98645732, 689500518476];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{
        circular_list::{CircularList, NodeRef, SinglyLinked},
        parse::single_digit,
    };
    use bare_metal_modulo::{MNum, OffsetNum};
    use itertools::Itertools;
    use nom::{multi::many1, Finish};
    use std::{collections::HashMap, marker::PhantomData};

    /// The labels for the cups.
    type Label = usize;

    /// Behavior specific to a particular part of the problem.
    pub trait Part {
        /// Adds any additional cups labels to the initially parsed labels.
        fn add_cups(cups: &mut Vec<Label>);
        /// Calculates the score starting at what should be the cup labeled 1.
        fn score(one: &NodeRef<SinglyLinked<Label>>) -> u64;
    }

    /// Behavior for part one.
    pub struct PartOne;
    impl Part for PartOne {
        fn add_cups(_cups: &mut Vec<Label>) {}

        fn score(one: &NodeRef<SinglyLinked<Label>>) -> u64 {
            one.iter(true)
                .skip(1)
                .map(|nr| nr.value().to_string())
                .collect::<String>()
                .parse()
                .unwrap()
        }
    }

    /// Behavior for part two.
    pub struct PartTwo;
    impl Part for PartTwo {
        fn add_cups(cups: &mut Vec<Label>) {
            for i in (cups.len() + 1)..=1000000 {
                cups.push(i);
            }
        }

        fn score(one: &NodeRef<SinglyLinked<Label>>) -> u64 {
            one.iter(true)
                .skip(1)
                .take(2)
                .map(|nr| u64::try_from(*nr.value()).unwrap())
                .product()
        }
    }

    /// A circle of cups, which can be parsed from text input.
    ///
    /// This is also an [`Iterator`] over the move numbers that the crab makes,
    /// with the arrangement of the cups changing accordingly.
    #[derive(Debug)]
    pub struct Cups<P: Part> {
        /// The circular list of cups labels.
        cups: CircularList<SinglyLinked<Label>>,
        /// Needed to have the [`Part`] parameter `P`.
        _phantom: PhantomData<P>,
    }
    impl<P: Part> Cups<P> {
        /// Parses the cups from text input.
        pub fn from_str(s: &str) -> AocResult<Self> {
            let mut cups: Vec<Label> = many1::<_, _, NomParseError, _>(single_digit)(s)
                .finish()
                .discard_input()?
                .into_iter()
                .map(|x| x.into())
                .collect();

            // Verify that we have enough cups
            if cups.len() < 4 {
                return Err(AocError::InvalidInput(
                    format!("Only found {} cups, which is not enough", cups.len()).into(),
                ));
            }

            // Ensure that the cups have consecutive labels starting with 1
            if cups.iter().copied().sorted().ne(1..=cups.len()) {
                return Err(AocError::InvalidInput(
                    format!(
                        "The {} cups do not have valid, consecutive labels",
                        cups.len()
                    )
                    .into(),
                ));
            }

            // Add additional cups based on the part
            P::add_cups(&mut cups);

            // Created owned slice
            let cups = CircularList::new(cups);

            Ok(Cups {
                cups,
                _phantom: PhantomData,
            })
        }

        /// Returns an [`Iterator`] over the moves of the game.
        ///
        /// Note that the first element of the iterator is *after* the first move, not
        /// the original state of the cups.
        /// The iterator yields the *new* current cup selected at the end of each
        /// move.
        pub fn start_game(&self) -> Game<P> {
            Game {
                list: &self.cups,
                lookup: self.cups.iter_const().map(|nr| (*nr.value(), nr)).collect(),
                current: self.cups.iter_const().next().unwrap(),
                _phantom: PhantomData,
            }
        }
    }

    /// An [`Iterator`] over the moves of the cup game.
    ///
    /// See [`Cups::start_game`] for more information.
    pub struct Game<'a, P: Part> {
        /// A reference to the circular list of cups, which will be mutated as
        /// the game proceeds.
        list: &'a CircularList<SinglyLinked<Label>>,
        /// A map of the labels to the cups.
        ///
        /// This is needed to speed things up for part two.
        lookup: HashMap<Label, NodeRef<'a, SinglyLinked<Label>>>,
        /// Reference to the current cup in the game.
        current: NodeRef<'a, SinglyLinked<Label>>,
        /// Needed to have the [`Part`] parameter `P`.
        _phantom: PhantomData<P>,
    }
    impl<'a, P: Part> Iterator for Game<'a, P> {
        type Item = NodeRef<'a, SinglyLinked<Label>>;

        fn next(&mut self) -> Option<Self::Item> {
            // First remove the next three cups
            let three = (0..3).map(|_| self.current.remove_next()).collect_vec();

            // Search for the destination cup
            let mut dest_label = OffsetNum::new(*self.current.value(), self.list.original_len(), 1);
            let mut dest = loop {
                // Decrement the destination cup and ensure it was not just picked up
                dest_label -= 1;
                if three.iter().all(|nr| *nr.value() != dest_label.a()) {
                    break self.lookup[&dest_label.a()].clone();
                }
            };

            // Insert the three cups back after the destination cup
            for nr in three {
                dest.insert_after(nr.clone());
                dest = nr;
            }

            // Lastly, select the new current cup
            self.current = self.current.next();
            Some(self.current.clone())
        }
    }
    impl<P: Part> Game<'_, P> {
        /// Calculates the score for the current arrangement of cups.
        pub fn score(&self) -> u64 {
            P::score(&self.lookup[&1])
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 23,
    name: "Crab Cups",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let cups: Cups<PartOne> = Cups::from_str(input.expect_input()?)?;
            let mut game = cups.start_game();

            // Process
            game.iterations(100);
            Ok(game.score().into())
        },
        // Part two
        |input| {
            // Generation
            let cups: Cups<PartTwo> = Cups::from_str(input.expect_input()?)?;
            let mut game = cups.start_game();

            // Process
            game.iterations(10000000);
            Ok(game.score().into())
        },
    ],
};
