use std::{
    cell::RefCell,
    collections::HashMap,
    convert::TryInto,
    fmt,
    rc::{Rc, Weak},
};

use itertools::Itertools;
use nom::{multi::many1, Finish};

use crate::aoc::{parse::single_digit, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expensive_test, solution_test};
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

#[derive(Clone, new)]
struct CupRef {
    rc: Weak<RefCell<Cup>>,
}
impl CupRef {
    fn label(&self) -> Label {
        self.rc.upgrade().unwrap().borrow().label
    }

    fn next(&self) -> Option<CupRef> {
        self.rc.upgrade().unwrap().borrow().next.as_ref().cloned()
    }

    fn set_next(&self, next: Option<CupRef>) -> Option<CupRef> {
        let rc = self.rc.upgrade().unwrap();
        let mut cup = rc.borrow_mut();
        let old = cup.next.take();
        cup.next = next;
        old
    }

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

#[derive(Debug)]
struct CupIter {
    first_label: Label,
    next_ref: Option<CupRef>,
}
impl CupIter {
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

type Label = u32;
#[derive(new)]
struct Cup {
    label: Label,
    next: Option<CupRef>,
}
impl fmt::Debug for Cup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.label.fmt(f)
    }
}

trait Part {
    fn add_cups(&self, _cups: &mut Vec<Label>) {}
    fn score(&self, one: &CupRef) -> u64;
}
struct PartA;
impl Part for PartA {
    fn score(&self, one: &CupRef) -> u64 {
        one.iter()
            .skip(1)
            .map(|cr| cr.label().to_string())
            .collect::<String>()
            .parse()
            .unwrap()
    }
}
struct PartB;
impl Part for PartB {
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

struct Cups {
    // NOTE: We need RefCell here to complete the circle
    cups: Box<[Rc<RefCell<Cup>>]>,
    // NOTE: This is needed to speed things up for part b)
    lookup: HashMap<Label, CupRef>,
    current: CupRef,
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
    fn from_str(s: &str, part: &dyn Part) -> AocResult<Self> {
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
        })
    }

    fn next(&mut self) {
        // First remove the next three cups
        let three = self.current.next().unwrap();
        self.current
            .set_next(three.iter().nth(2).unwrap().set_next(None));

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

    fn run(&mut self, iterations: usize) {
        for _i in 0..iterations {
            //println!("Move {}", _i + 1);
            self.next();
        }
    }

    fn score(&self, part: &dyn Part) -> u64 {
        part.score(&self.lookup[&1])
    }
}

pub const SOLUTION: Solution = Solution {
    day: 23,
    name: "Crab Cups",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let part = &PartA;
            let mut cups = Cups::from_str(input.expect_input()?, part)?;
            cups.run(100);

            // Process
            Ok(cups.score(part).into())
        },
        // Part b)
        |input| {
            // Generation
            let part = &PartB;
            let mut cups = Cups::from_str(input.expect_input()?, part)?;
            cups.run(10000000);

            // Process
            Ok(cups.score(part).into())
        },
    ],
};
