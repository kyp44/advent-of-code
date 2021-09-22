use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(786240), Unsigned(831600)],
    "5000",
    vec![180u64, 168].answer_vec()
    }
}

struct Delivery {
    target: usize,
    presents: Vec<usize>,
}
impl Delivery {
    fn new(target: usize, present_mult: usize, house_limit: Option<usize>) -> Self {
        // Maximum house for which we need to compute the number of presents
        // as this is garanteed to be above the target.
        // This was derived from the lowest even house number n such that:
        // target <= 10*(n + n/2)
        // since n/2 will be a divisor for even n.
        let mut max = 2 * target / 30;
        max += if max % 2 == 0 { 2 } else { 1 };

        // We implement a seive that calculates all number of presents
        // (effectively the sum of divisors) for all numbers up to max.
        let mut presents = vec![0; max];
        // Each elf
        for i in 1..=max {
            let mut count = 0;
            let mut j = i;
            loop {
                presents[j - 1] += present_mult * i;
                j += i;
                if j > max {
                    break;
                }
                count += 1;
                if let Some(l) = house_limit {
                    if count > l {
                        break;
                    }
                }
            }
        }

        Delivery { target, presents }
    }

    fn first_house(&self) -> AocResult<u64> {
        Ok((self
            .presents
            .iter()
            .position(|p| *p >= self.target)
            .ok_or_else(|| AocError::Process("No solution found!".into()))?
            + 1)
        .try_into()
        .unwrap())
    }
}

pub const SOLUTION: Solution = Solution {
    day: 20,
    name: "Infinite Elves and Infinite Houses",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let delivery = Delivery::new(usize::from_str(input)?, 10, None);

            // Process
            /*for h in 1..10 {
                println!("House {}: {}", h, delivery.presents[h - 1]);
            }*/

            Ok(delivery.first_house()?.into())
        },
        // Part b)
        |input| {
            // Generation
            let delivery = Delivery::new(usize::from_str(input)?, 11, Some(50));

            // Process
            /*for h in 1..30 {
                println!("House {}: {}", h, delivery.presents[h - 1]);
            }*/

            Ok(delivery.first_house()?.into())
        },
    ],
};
