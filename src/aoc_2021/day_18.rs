use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(123)],
    "[1,1]
[2,2]
[3,3]
[4,4]",
    vec![11u64].answer_vec()
    }
}

enum Number {
    Num(u8),
    Pair(Box<(Number, Number)>),
}
impl Parseable<'_> for Number {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        todo!()
    }
}
impl Number {
    fn reduce(self) -> Self {
        loop {}
    }
}

pub const SOLUTION: Solution = Solution {
    day: 18,
    name: "Snailfish",
    solvers: &[
        // Part a)
        |input| {
            // Generation

            // Process
            Ok(0u64.into())
        },
    ],
};
