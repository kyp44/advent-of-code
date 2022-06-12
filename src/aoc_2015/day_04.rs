use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(254575), Unsigned(1038736)],
    "abcdef",
    vec![609043u64].answer_vec(),
    "pqrstuv",
    vec![1048970u64].answer_vec()
    }
}

trait Part {
    fn check_third_byte(byte: u8) -> bool;
}
struct PartA;
impl Part for PartA {
    fn check_third_byte(byte: u8) -> bool {
        byte & 0xF0 == 0
    }
}
struct PartB;
impl Part for PartB {
    fn check_third_byte(byte: u8) -> bool {
        byte == 0
    }
}

fn solve<P: Part>(input: &str) -> u64 {
    let input = input.trim();

    let mut ans: u64 = 0;
    loop {
        let hash = md5::compute(format!("{}{}", input, ans));

        // Check that the first hex digits are zero
        if hash[0] == 0 && hash[1] == 0 && P::check_third_byte(hash[2]) {
            break ans;
        }
        ans += 1;
    }
}

pub const SOLUTION: Solution = Solution {
    day: 4,
    name: "The Ideal Stocking Stuffer",
    preprocessor: None,
    solvers: &[
        // Part a)
        |input| Ok(solve::<PartA>(input.expect_input()?).into()),
        // Part b)
        |input| Ok(solve::<PartB>(input.expect_input()?).into()),
    ],
};
