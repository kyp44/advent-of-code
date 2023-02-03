use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::{expensive_test, solution_test};
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(98645732), Unsigned(689500518476)],
    "389125467",
    vec![Some(Unsigned(67384529)), None]
    }

    expensive_test! {
    "389125467",
    vec![None, Some(Unsigned(149245887792))]
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::single_digit;
    use derive_new::new;
    use itertools::Itertools;
    use nom::{multi::many1, Finish};
    use std::{
        cell::RefCell,
        collections::HashMap,
        convert::TryInto,
        fmt,
        rc::{Rc, Weak},
    };

    /// Shared reference to a [Cup].
    ///
    /// Weak references are needed here to avoid the issue that circular strong references
    /// that would lead to a memory leak (one of the few ways to create a memory
    /// leak in safe Rust).
    #[derive(Clone, new)]
    pub struct CupRef {
        /// The weak reference to the cup.
        rc: Weak<RefCell<Cup>>,
    }
    impl CupRef {
        /// The label for the referent cup.
        fn label(&self) -> Label {
            self.rc.upgrade().unwrap().borrow().label
        }

        /// A reference to the next cup in the chain of cups, if there is one.
        fn next(&self) -> Option<CupRef> {
            self.rc.upgrade().unwrap().borrow().next.as_ref().cloned()
        }

        /// Sets the optional next cup reference for the referent cup,
        /// returning a reference to the previously set next cup if
        /// there was one.
        fn set_next(&self, next: Option<CupRef>) -> Option<CupRef> {
            let rc = self.rc.upgrade().unwrap();
            let mut cup = rc.borrow_mut();
            let old = cup.next.take();
            cup.next = next;
            old
        }

        /// Returns an [Iterator] over the chain of cups with the first element being this cup.
        fn iter(&self) -> CupIter {
            CupIter::new(self)
        }
    }
    impl From<&Rc<RefCell<Cup>>> for CupRef {
        fn from(rc: &Rc<RefCell<Cup>>) -> Self {
            CupRef::new(Rc::downgrade(rc))
        }
    }
    impl fmt::Debug for CupRef {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.rc.upgrade().unwrap().borrow().fmt(f)
        }
    }

    /// [Iterator] over a chain of cups.
    ///
    /// If the chain is circular the [Iterator] will stop just before the
    /// first cup is reached again so that every cup in the circle is visited
    /// exactly once.
    #[derive(Debug)]
    struct CupIter {
        /// The label for the first cup, which is used know when to stop.
        first_label: Label,
        /// Reference to the cup that will be returned next, if any.
        next_ref: Option<CupRef>,
    }
    impl CupIter {
        /// Creates a new [Iterator] with the first item being the passed cup.
        fn new(cr: &CupRef) -> CupIter {
            CupIter {
                first_label: cr.label(),
                next_ref: Some(cr.clone()),
            }
        }
    }
    impl Iterator for CupIter {
        type Item = CupRef;

        fn next(&mut self) -> Option<Self::Item> {
            let out = self.next_ref.take();

            if let Some(curr) = &out {
                self.next_ref = curr.next();
                if let Some(next) = &self.next_ref {
                    if next.label() == self.first_label {
                        self.next_ref = None;
                    }
                }
            }

            out
        }
    }

    /// The labels for the cups.
    type Label = u32;

    /// A cup.
    #[derive(new)]
    pub struct Cup {
        /// Label for the cup.
        label: Label,
        /// A reference to the next cup in the chain, if any.
        next: Option<CupRef>,
    }
    impl fmt::Debug for Cup {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.label.fmt(f)
        }
    }

    /// Behavior specific to a particular part of the problem.
    pub trait Part {
        /// Add any additional cups labels to the initially parsed labels.
        fn add_cups(&self, _cups: &mut Vec<Label>);
        /// Calculate the score starting at what should be the cup labeled 1.
        fn score(&self, one: &CupRef) -> u64;
    }

    /// Behavior for part one.
    pub struct PartOne;
    impl Part for PartOne {
        fn add_cups(&self, _cups: &mut Vec<Label>) {}

        fn score(&self, one: &CupRef) -> u64 {
            one.iter()
                .skip(1)
                .map(|cr| cr.label().to_string())
                .collect::<String>()
                .parse()
                .unwrap()
        }
    }

    /// Behavior for part two.
    pub struct PartTwo;
    impl Part for PartTwo {
        fn add_cups(&self, cups: &mut Vec<Label>) {
            for i in (cups.len() + 1)..=1000000 {
                cups.push(i.try_into().unwrap());
            }
        }

        fn score(&self, one: &CupRef) -> u64 {
            one.iter()
                .skip(1)
                .take(2)
                .map(|cr| u64::from(cr.label()))
                .product()
        }
    }

    /// A circle of cups, which can be parsed from text input.
    ///
    /// This is also an [Iterator] over the move numbers that the crab makes,
    /// with the arrangement of the cups changing accordingly.
    pub struct Cups {
        /// All the cups with strong references so that this is effectively their owner.
        ///
        /// NOTE: We need the [RefCell] here to complete the circle.
        cups: Box<[Rc<RefCell<Cup>>]>,
        /// A map of the labels to the cups.
        ///
        /// NOTE: This is needed to speed things up for part two.
        lookup: HashMap<Label, CupRef>,
        /// Reference to the current cup in the game.
        current: CupRef,
        /// The next move number.
        next_move: usize,
    }
    impl fmt::Debug for Cups {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                (self.current.iter().map(|cr| format!("{cr:?}")).join(", "))
            )
        }
    }
    impl Cups {
        /// Parse the cups from text input.
        pub fn from_str(s: &str, part: &dyn Part) -> AocResult<Self> {
            let mut cups = many1::<_, _, NomParseError, _>(single_digit)(s)
                .finish()
                .discard_input()?;

            // Verify that we have enough cups
            if cups.len() < 4 {
                return Err(AocError::InvalidInput(
                    format!("Only found {} cups, which is not enough", cups.len()).into(),
                ));
            }

            // Ensure that the cups have consecutive labels starting with 1
            if cups
                .iter()
                .map(|l| -> usize { (*l).try_into().unwrap() })
                .sorted()
                .ne(1..=cups.len())
            {
                return Err(AocError::InvalidInput(
                    format!(
                        "The {} cups do not have valid, consecutive labels",
                        cups.len()
                    )
                    .into(),
                ));
            }

            // Add additional cups based on the part
            part.add_cups(&mut cups);

            // Created owned slice
            let cups = cups
                .into_iter()
                .map(|l| Rc::new(RefCell::new(Cup::new(l, None))))
                .collect_vec()
                .into_boxed_slice();

            // Create lookup table
            let lookup: HashMap<Label, CupRef> = cups
                .iter()
                .map(|rc| (rc.borrow().label, CupRef::from(rc)))
                .collect();

            // Now create circle of references
            for win in cups.windows(2) {
                win[0].borrow_mut().next = Some(CupRef::from(&win[1]));
            }
            cups.last().unwrap().borrow_mut().next = Some(CupRef::from(&cups[0]));
            let current = CupRef::from(&cups[0]);

            Ok(Cups {
                cups,
                lookup,
                current,
                next_move: 0,
            })
        }

        /// Calculates the score for the current arrangement of cups.
        pub fn score(&self, part: &dyn Part) -> u64 {
            part.score(&self.lookup[&1])
        }
    }
    impl Iterator for Cups {
        type Item = usize;

        fn next(&mut self) -> Option<Self::Item> {
            let next_move = self.next_move;

            if next_move > 0 {
                // First remove the next three cups
                let three = self.current.next().unwrap();
                self.current
                    .set_next(three.iter().iterations(3).unwrap().set_next(None));

                // Search for the destination cup
                let mut dest_label = self.current.label();
                let len: Label = self.cups.len().try_into().unwrap();
                let dest = loop {
                    dest_label = ((dest_label + len - 2) % len) + 1;
                    if three.iter().all(|cr| cr.label() != dest_label) {
                        break &self.lookup[&dest_label];
                    }
                };

                // Insert the three cups back after the destination cup
                three.iter().last().unwrap().set_next(dest.next());
                dest.set_next(Some(three));

                // Lastly, select the new current cup
                self.current = self.current.next().unwrap();
            }

            self.next_move += 1;
            Some(next_move)
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
            let part = &PartOne;
            let mut cups = Cups::from_str(input.expect_input()?, part)?;

            // Process
            cups.find(|m| *m == 100);
            Ok(cups.score(part).into())
        },
        // Part two
        |input| {
            // Generation
            let part = &PartTwo;
            let mut cups = Cups::from_str(input.expect_input()?, part)?;

            // Process
            cups.find(|m| *m == 10000000);
            Ok(cups.score(part).into())
        },
    ],
};
