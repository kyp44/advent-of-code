use std::{cell::RefCell, rc::Rc};

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

struct CupRef {
    rc: Rc<RefCell<Cup>>,
}
impl CupRef {
    fn label(&self) -> u8 {
        self.rc.borrow().label
    }

    fn set_next(&self, new: Option<CupRef>) -> Option<CupRef> {
        todo!()
    }
}

struct Cup {
    label: u8,
    next: Option<CupRef>,
}
/*
impl Cup {
    fn new_ref(label: u8, next: Option<CupRef>) -> CupRef {
        Rc::new(RefCell::new(Cup { label, next }))
    }
}

struct Cups {
    current: CupRef,
}
impl Cups {
    fn new() -> Self {
        let mut c1 = Cup::new_ref(1, None);
        let c2 = Cup::new_ref(1, c1.clone().into());
    c2.set_next();

        Cups { current: c1 }
    }
}*/

pub const SOLUTION: Solution = Solution {
    day: 23,
    name: "Crab Cups",
    solvers: &[
        // Part a)
        |input| {
            // Generation

            // Process
            Ok(0.into())
        },
    ],
};
