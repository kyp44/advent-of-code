use crate::aoc::{ParseResult, Parseable, Solution};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![],
    "0: 4 1 5
1: 2 3 | 3 2
2: 4 4 | 5 5
3: 4 5 | 5 4
4: \"a\"
5: \"b\"

ababbb
bababa
abbbab
aaabbb
aaaabbb",
    vec![Some(2)]
    }
}

enum Rule<'a> {
    Match(&'a str),
    Seq(Vec<Vec<usize>>),
}
impl<'a> Parseable<'a> for Rule<'a> {
    fn parser(input: &'a str) -> ParseResult<Self> {
	let quote = "\"";
        alt((
	    map(trim(delimited(tag(quote), is_not(quote), tag(quote))), |s| Rule::Match(s)),
	    map(
	))(input)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 19,
    name: "Monster Messages",
    solvers: &[
        // Part a)
        |input| {
            // Generation

            // Process
            Ok(0)
        },
    ],
};
