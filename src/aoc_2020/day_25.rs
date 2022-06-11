use std::str::FromStr;

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
            vec![Unsigned(6421487)],
        "5764801
    17807724",
    vec![14897079u64].answer_vec()
    }
}

#[derive(new)]
struct Transform {
    subject: u64,
    #[new(value = "1")]
    value: u64,
}
impl Iterator for Transform {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.value *= self.subject;
        self.value %= 20201227;
        Some(self.value)
    }
}

#[derive(Debug)]
struct Problem {
    card_key: u64,
    door_key: u64,
}
impl FromStr for Problem {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = u64::gather(s.lines())?;

        if v.len() != 2 {
            return Err(AocError::InvalidInput(
                format!(
                    "Input must contain exactly two numbers but found {}",
                    v.len(),
                )
                .into(),
            ));
        }

        Ok(Problem {
            card_key: v[0],
            door_key: v[1],
        })
    }
}
impl Problem {
    fn solve(&self) -> AocResult<u64> {
        // First determine the secret loop size for each
        fn find_loop_size(key: u64) -> usize {
            Transform::new(7).take_while(|v| *v != key).count() + 1
        }

        let card_loop = find_loop_size(self.card_key);
        let door_loop = find_loop_size(self.door_key);

        // Now calculate the encryption key
        fn transform(subject: u64, loop_size: usize) -> u64 {
            Transform::new(subject).nth(loop_size - 1).unwrap()
        }
        let enc_key = transform(self.door_key, card_loop);
        if enc_key != transform(self.card_key, door_loop) {
            return Err(AocError::Process(
                "The encryption keys do not match!".into(),
            ));
        }

        Ok(enc_key)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 25,
    name: "Combo Breaker",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let problem: Problem = input.expect_input()?.parse()?;

            // Process
            Ok(problem.solve()?.into())
        },
    ],
};
