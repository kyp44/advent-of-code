use std::{
    cell::RefCell,
    convert::TryInto,
    fmt,
    rc::{Rc, Weak},
    str::FromStr,
};

use itertools::Itertools;
use nom::{multi::many1, Finish};

use crate::aoc::{prelude::*, single_digit};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Number;

    solution_test! {
    vec![Number(98645732)],
    "389125467",
    vec![67384529].answer_vec()
    }
}

#[derive(Clone)]
struct CupRef {
    rc: Weak<RefCell<Cup>>,
}
impl CupRef {
    fn new(rc: Weak<RefCell<Cup>>) -> Self {
        CupRef { rc }
    }

    fn label(&self) -> u8 {
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

struct CupIter {
    first_label: u8,
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

struct Cup {
    label: u8,
    next: Option<CupRef>,
}
impl Cup {
    fn new(label: u8, next: Option<CupRef>) -> Cup {
        Cup { label, next }
    }
}
impl fmt::Debug for Cup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.label.fmt(f)
    }
}

struct Cups {
    // NOTE: We need RefCell here to complete the circle
    cups: Box<[Rc<RefCell<Cup>>]>,
    current: CupRef,
}
impl FromStr for Cups {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lv = many1::<_, _, NomParseError, _>(single_digit)(s)
            .finish()
            .discard_input()?;

        // Verify that we have enough cups
        if lv.len() < 4 {
            return Err(AocError::InvalidInput(format!(
                "Only found {} cups, which is not enough",
                lv.len()
            )));
        }

        // Created owned slice
        let cups = lv
            .into_iter()
            .map(|l| Rc::new(RefCell::new(Cup::new(l.try_into().unwrap(), None))))
            .collect_vec()
            .into_boxed_slice();

        // Now create circle of references
        for win in cups.windows(2) {
            win[0].borrow_mut().next = Some(CupRef::from(&win[1]));
        }
        cups.last().unwrap().borrow_mut().next = Some(CupRef::from(&cups[0]));
        let current = CupRef::from(&cups[0]);

        // Lastly, ensure that the cups have consecutive labels starting with 1
        let mut labels: Vec<usize> = current.iter().map(|cr| cr.label().into()).collect();
        labels.sort_unstable();
        if labels != (1..=cups.len()).collect::<Vec<usize>>() {
            return Err(AocError::InvalidInput(format!(
                "The {} cups do not have valid, consecutive labels",
                cups.len()
            )));
        }

        Ok(Cups { cups, current })
    }
}
impl fmt::Debug for Cups {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            (self.current.iter().map(|cr| format!("{:?}", cr)).join(", "))
        )
    }
}
impl Cups {
    fn next(&mut self) {
        // First remove the next three cups
        let three = self.current.next().unwrap();
        self.current
            .set_next(three.iter().nth(2).unwrap().set_next(None));

        // Search for the destination cup
        let mut dest_label = self.current.label();
        let len: u8 = self.cups.len().try_into().unwrap();
        let dest = loop {
            dest_label = ((dest_label + len - 2) % len) + 1;
            if let Some(cr) = self.current.iter().find(|cr| cr.label() == dest_label) {
                break cr;
            }
        };

        // Insert the three cups back after the destination cup
        three.iter().last().unwrap().set_next(dest.next());
        dest.set_next(Some(three));

        // Lastly, select the new current cup
        self.current = self.current.next().unwrap();
    }

    fn run(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.next();
        }
    }

    fn score(&self) -> u64 {
        let one = self.current.iter().find(|cr| cr.label() == 1).unwrap();
        one.iter()
            .skip(1)
            .map(|cr| cr.label().to_string())
            .collect::<String>()
            .parse()
            .unwrap()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 23,
    name: "Crab Cups",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let mut cups = Cups::from_str(input)?;
            cups.run(100);

            // Process
            Ok(cups.score().into())
        },
    ],
};
