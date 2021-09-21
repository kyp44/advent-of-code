use crate::aoc::prelude::*;
use num::{Integer, One};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![],
    "500",
    vec![24u64].answer_vec()
    }
}

struct Factors<T> {
    n: T,
    factor: T,
}
impl<T> Factors<T>
where
    T: One,
{
    fn new(n: T) -> Self {
        Factors {
            n,
            factor: T::one(),
        }
    }
}
impl<T> Iterator for Factors<T>
where
    T: Integer + Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let factor = self.factor;
            if factor > self.n {
                break None;
            } else if factor > self.n.div_floor(&(T::one() + T::one())) {
                self.factor = self.n + T::one();
                break Some(self.n);
            } else {
                self.factor = self.factor + T::one();
                if self.n.is_multiple_of(&factor) {
                    break Some(factor);
                }
            }
        }
    }
}

pub const SOLUTION: Solution = Solution {
    day: 20,
    name: "Infinite Elves and Infinite Houses",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let target: u64 = u64::from_str(input)?;
            let mut house = 100000;
            loop {
                let presents = Factors::new(house).sum::<u64>() * 10;
                println!("House {}: {}", house, presents);
                if presents >= target {
                    break;
                }
                house += 1;
            }

            // Process
            Ok(house.into())
        },
    ],
};
