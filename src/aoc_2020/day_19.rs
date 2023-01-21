use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(149), Unsigned(332)],
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
    vec![Some(Unsigned(2)), None],
    "42: 9 14 | 10 1
9: 14 27 | 1 26
10: 23 14 | 28 1
1: \"a\"
11: 42 31
5: 1 14 | 15 1
19: 14 1 | 14 14
12: 24 14 | 19 1
16: 15 1 | 14 14
31: 14 17 | 1 13
6: 14 14 | 1 14
2: 1 24 | 14 4
0: 8 11
13: 14 3 | 1 12
15: 1 | 14
17: 14 2 | 1 7
23: 25 1 | 22 14
28: 16 1
4: 1 1
20: 14 14 | 1 15
3: 5 14 | 16 1
27: 1 6 | 14 18
14: \"b\"
21: 14 1 | 1 14
25: 1 1 | 1 14
22: 14 14
8: 42
26: 14 22 | 1 20
18: 15 15
7: 14 5 | 1 21
24: 14 1

abbbbbabbbaaaababbaabbbbabababbbabbbbbbabaaaa
bbabbbbaabaabba
babbbbaabbbbbabbbbbbaabaaabaaa
aaabbbbbbaaaabaababaabababbabaaabbababababaaa
bbbbbbbaaaabbbbaaabbabaaa
bbbababbbbaaaaaaaabbababaaababaabab
ababaaaaaabaaab
ababaaaaabbbaba
baabbaaaabbaaaababbaababb
abbbbabbbbaaaababbbbbbaaaababb
aaaaabbaabaaaaababaa
aaaabbaaaabbaaa
aaaabbaabbaaaaaaabbbabbbaaabbaabaaa
babaaabbbaaabaababbaabababaaab
aabbbbbaabbbaaaaaabbbbbababaaaaabbaaabba",
    vec![3u64, 12].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use crate::aoc::parse::trim;
    use itertools::process_results;
    use nom::{
        branch::alt,
        bytes::complete::{is_not, tag},
        character::complete::space1,
        combinator::map,
        multi::{many1, separated_list1},
        sequence::{delimited, preceded, separated_pair},
        Finish,
    };
    use std::collections::HashMap;
    use std::convert::TryInto;

    /// A single rule, which can be parsed from text input.
    #[derive(Debug)]
    pub enum Rule<'a> {
        /// Rule in which the portion must match an explicit string.
        Match(&'a str),
        /// Rule in which the portion must match one of a list sequences of other rules.
        Seq(Vec<Vec<usize>>),
    }
    impl<'a> Parseable<'a> for Rule<'a> {
        fn parser(input: &'a str) -> NomParseResult<&str, Self> {
            let quote = "\"";
            alt((
                map(
                    trim(false, delimited(tag(quote), is_not(quote), tag(quote))),
                    Rule::Match,
                ),
                map(
                    separated_list1(
                        preceded(space1, tag("|")),
                        many1(preceded(space1, nom::character::complete::u32)),
                    ),
                    |v| {
                        Rule::Seq(
                            v.into_iter()
                                .map(|iv| iv.into_iter().map(|d| d.try_into().unwrap()).collect())
                                .collect(),
                        )
                    },
                ),
            ))(input)
        }
    }

    /// Behavior specific to one particular part of the problem.
    pub trait Part {
        /// Fixes a rule by possibly substituting it with another.
        fn fix_rule<'a>(&self, num: usize, rule: Rule<'a>) -> Rule<'a>;
    }

    /// Behavior for part one.
    pub struct PartOne;
    impl Part for PartOne {
        fn fix_rule<'a>(&self, _num: usize, rule: Rule<'a>) -> Rule<'a> {
            // There are no substitutions so just return the input rule.
            rule
        }
    }

    /// Behavior for part two.
    pub struct PartTwo;
    impl Part for PartTwo {
        fn fix_rule<'a>(&self, num: usize, rule: Rule<'a>) -> Rule<'a> {
            match num {
                8 => Rule::Seq(vec![vec![42], vec![42, 8]]),
                11 => Rule::Seq(vec![vec![42, 31], vec![42, 11, 31]]),
                _ => rule,
            }
        }
    }

    /// A set of rules, which can be parsed from text input.
    #[derive(Debug)]
    struct RuleSet<'a> {
        /// Map of rule numbers to their rules.
        rules: HashMap<usize, Rule<'a>>,
    }
    impl<'a> RuleSet<'a> {
        /// Parse from text input, using rule replacements defined in `part`.
        fn from_str(s: &'a str, part: &dyn Part) -> AocResult<Self> {
            let mut rules = HashMap::new();
            for line in s.lines() {
                let (n, rule) =
                    separated_pair(nom::character::complete::u32, tag(":"), Rule::parser)(line)
                        .finish()
                        .discard_input()?;
                let n: usize = n.try_into().unwrap();
                if rules.insert(n, part.fix_rule(n, rule)).is_some() {
                    return Err(AocError::InvalidInput(
                        format!("There is a duplicate for rule {n}").into(),
                    ));
                }
            }
            Ok(RuleSet { rules })
        }

        /// Determined whether the input string is valid according to a particular rule in the set.
        fn is_valid(&self, s: &str, rule_num: usize) -> AocResult<bool> {
            /// Recursive internal function for [RuleSet::is_valid].
            fn valid<'a>(
                rule_set: &RuleSet,
                s: &'a str,
                rule_num: usize,
                level: usize,
            ) -> AocResult<(bool, &'a str)> {
                let _tab: String = (0..level).map(|_| "  ").collect();
                let rule = rule_set.rules.get(&rule_num).ok_or_else(|| {
                    AocError::Process(format!("Rule {rule_num} not found").into())
                })?;
                let mut matched = true;
                let mut remaining = s;
                /*println!(
                    "{}Rule {}: Checking that '{}' starts with rule {:?} {{",
                    _tab, rule_num, s, rule,
                );*/

                match rule {
                    Rule::Match(ms) => {
                        if remaining.starts_with(ms) {
                            remaining = &s[ms.len()..];
                        } else {
                            matched = false;
                        }
                    }
                    Rule::Seq(ov) => {
                        for mv in ov.iter() {
                            let mut last_rn = rule_num;
                            let mut seq_rem = remaining;
                            matched = true;

                            for nrn in mv.iter() {
                                // Have we run out of string?
                                if seq_rem.is_empty() {
                                    // Apparently we disallow partial pattern mattern unless
                                    // the partial match ended on a looped rule
                                    matched = last_rn == rule_num;
                                    break;
                                }
                                (matched, seq_rem) = valid(rule_set, seq_rem, *nrn, level + 1)?;
                                if !matched {
                                    break;
                                }
                                last_rn = *nrn;
                            }
                            if matched {
                                remaining = seq_rem;
                                break;
                            }
                        }
                    }
                }
                /*println!(
                    "{}}} Matched: {}, Remaining: '{}'",
                    _tab, matched, remaining
                );*/
                Ok((matched, remaining))
            }

            // Must have matched the entire string
            let (matched, remaining) = valid(self, s, rule_num, 0)?;
            if remaining.is_empty() {
                return Ok(matched);
            }
            Ok(false)
        }
    }

    /// Problem definition, which can be parsed from text input.
    #[derive(Debug)]
    pub struct Problem<'a> {
        /// Rule set for the problem.
        rule_set: RuleSet<'a>,
        /// List of strings to validate.
        strings: Vec<&'a str>,
    }
    impl<'a> Problem<'a> {
        /// Parse from text input, using rule replacements defined in `part`.
        pub fn from_str(s: &'a str, part: &dyn Part) -> AocResult<Self> {
            let secs = s.sections(2)?;
            Ok(Problem {
                rule_set: RuleSet::from_str(secs[0], part)?,
                strings: secs[1].lines().collect(),
            })
        }

        /// Count the strings that are valid according to rule 0 in the set.
        pub fn count_valid(&self) -> AocResult<u64> {
            process_results(
                self.strings.iter().map(|s| self.rule_set.is_valid(s, 0)),
                |iter| iter.filter_count(|valid| *valid),
            )
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 19,
    name: "Monster Messages",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let problem = Problem::from_str(input.expect_input()?, &PartOne)?;

            // Process
            Ok(problem.count_valid()?.into())
        },
        // Part two
        |input| {
            // Generation
            let problem = Problem::from_str(input.expect_input()?, &PartTwo)?;

            // Process
            Ok(problem.count_valid()?.into())
        },
    ],
};
