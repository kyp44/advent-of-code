use std::{borrow::BorrowMut, cell::RefCell, rc::Rc, str::FromStr};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Number;

    solution_test! {
    vec![],
    "389125467",
    vec![67384529].answer_vec()
    }
}

#[derive(Clone)]
struct CupRef {
    rc: Rc<RefCell<Cup>>,
}
impl CupRef {
    fn new(rc: Rc<RefCell<Cup>>) -> Self {
        CupRef { rc }
    }

    fn label(&self) -> u8 {
        self.rc.borrow().label
    }

    fn set_next(&self, new: Option<CupRef>) -> Option<CupRef> {
        let mut cup = (*self.rc).borrow_mut();
        let old = cup.next.take();
        cup.next = new;
        old
    }
}

struct Cup {
    label: u8,
    next: Option<CupRef>,
}
impl Cup {
    fn new_ref(label: u8, next: Option<CupRef>) -> CupRef {
        CupRef::new(Rc::new(RefCell::new(Cup { label, next })))
    }
}

struct Cups {
    current: CupRef,
}
impl FromStr for Cups {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {}
}
impl Cups {
    fn new() -> Self {
        let c1 = Cup::new_ref(1, None);
        let c2 = Cup::new_ref(1, c1.clone().into());
        c2.set_next(c1.clone().into());

        Cups { current: c1 }
    }
}

pub const SOLUTION: Solution = Solution {
    day: 23,
    name: "Crab Cups",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let cups = Cups::new();

            // Process
            Ok(0.into())
        },
    ],
};
