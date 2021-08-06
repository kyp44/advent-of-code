use std::{
    borrow::BorrowMut,
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
    vec![],
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
}
impl From<&Rc<RefCell<Cup>>> for CupRef {
    fn from(rc: &Rc<RefCell<Cup>>) -> Self {
        CupRef::new(Rc::downgrade(rc))
    }
}
impl fmt::Debug for CupRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let opt = self.rc.upgrade();
        match opt {
            Some(rc) => rc.borrow().label.fmt(f),
            None => opt.fmt(f),
        }
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
        write!(f, "{} -> ", self.label)?;
        match &self.next {
            Some(cr) => cr.fmt(f),
            None => self.next.fmt(f),
        }
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
            todo!()
        }
        /*
        for i in (1..cups.len()).rev() {
            /*println!(
                "{} {} {} {}",
                i,
                cups[i].label,
                Rc::strong_count(&cups[i]),
                Rc::weak_count(&cups[i])
            );*/
            Rc::get_mut(&mut cups[i - 1]).unwrap().next = Some((&cups[i]).into());
        }
        Rc::get_mut(cups.last_mut().unwrap()).unwrap().next = Some((&cups[0]).into());*/
        println!("{:?}", cups);

        /*Ok(Cups {
                cups,
                current: CupRef::from(&cups[0]),
        })*/
        todo!()
    }
}
impl Cups {
    /*fn new() -> Self {
        let c1 = Cup::new_ref(1, None);
        let c2 = Cup::new_ref(1, c1.clone().into());
        c2.set_next(c1.clone().into());

        Cups { current: c1 }
    }*/
}

pub const SOLUTION: Solution = Solution {
    day: 23,
    name: "Crab Cups",
    solvers: &[
        // Part a)
        |input| {
            println!("TODO: {}", input);
            // Generation
            let cups = Cups::from_str(input)?;

            // Process
            Ok(0.into())
        },
    ],
};
