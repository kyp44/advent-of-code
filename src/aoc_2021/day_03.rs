use bitvec::prelude::BitVec;

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(1)],
    "00100
11110
10110
10111
10101
01111
00111
11100
10000
11001
00010
01010",
    vec![198u64].answer_vec()
    }
}

impl Parseable<'_> for BitVec {
    fn parser(input: &str) -> NomParseResult<Self> {
        todo!()
    }
}

pub const SOLUTION: Solution = Solution {
    day: 3,
    name: "Binary Diagnostic",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            //let x = BitVec::gather(input);
            // TODO
            println!("{}", u16::from_str_radix("11110", 2).unwrap());

            // Process
            Ok(0u64.into())
        },
    ],
};
