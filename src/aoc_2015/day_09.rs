use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![],
    "London to Dublin = 464
London to Belfast = 518
Dublin to Belfast = 141",
    vec![605u64].answer_vec()
    }
}

#[derive(Eq, Debug)]
struct City {
    id: usize,
    name: String,
}
impl PartialEq for City {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
static CITY_ID: AtomicUsize = AtomicUsize::new(0);
impl City {
    fn new(name: &str) -> Self {
        City {
            id: CITY_ID.fetch_add(1, Ordering::Relaxed),
            name: name.to_string(),
        }
    }
}

struct Distance {
    source: Rc<City>,
    dest: Rc<City>,
    distance: u64,
}

struct Problem {
    cities: Box<[Rc<City>]>,
}

pub const SOLUTION: Solution = Solution {
    day: 9,
    name: "All in a Single Night",
    solvers: &[
        // Part a)
        |input| {
            // Generation

            // Process
            Ok(0u64.into())
        },
    ],
};
