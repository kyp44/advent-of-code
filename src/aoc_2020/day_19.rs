use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{digit1, space1},
    combinator::map,
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, separated_pair},
    Finish,
};

use crate::aoc::{
    trim, AocError, AocResult, DiscardInput, ParseResult, Parseable, Sections, Solution,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![149],
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
    vec![Some(2), None],
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
    vec![Some(3), Some(12)]
    }
}

#[derive(Debug)]
enum Rule<'a> {
    Match(&'a str),
    Seq(Vec<Vec<usize>>),
}
impl<'a> Parseable<'a> for Rule<'a> {
    fn parser(input: &'a str) -> ParseResult<Self> {
        let quote = "\"";
        alt((
            map(
                trim(delimited(tag(quote), is_not(quote), tag(quote))),
                |s| Rule::Match(s),
            ),
            map(
                separated_list1(preceded(space1, tag("|")), many1(preceded(space1, digit1))),
                |v: Vec<Vec<&str>>| {
                    Rule::Seq(
                        v.into_iter()
                            .map(|iv| {
                                iv.into_iter()
                                    .map(|ds| ds.parse::<usize>().unwrap())
                                    .collect()
                            })
                            .collect(),
                    )
                },
            ),
        ))(input)
    }
}

trait Part {
    fn fix_rule(_num: usize, rule: Rule) -> Rule {
        rule
    }
}
struct PartA;
impl Part for PartA {}
struct PartB;
impl Part for PartB {
    fn fix_rule(num: usize, rule: Rule) -> Rule {
        match num {
            8 => Rule::Seq(vec![vec![42], vec![42, 8]]),
            11 => Rule::Seq(vec![vec![42, 31], vec![42, 11, 31]]),
            _ => rule,
        }
    }
}

#[derive(Debug)]
struct RuleSet<'a> {
    rules: HashMap<usize, Rule<'a>>,
}
impl<'a> RuleSet<'a> {
    fn from_str<P: Part>(s: &'a str) -> AocResult<Self> {
        let mut rules = HashMap::new();
        for line in s.lines() {
            let (ns, rule) = separated_pair(digit1, tag(":"), Rule::parser)(line)
                .finish()
                .discard_input()?;
            let n: usize = ns.parse().unwrap();
            if let Some(_) = rules.insert(n, P::fix_rule(n, rule)) {
                return Err(AocError::InvalidInput(format!(
                    "There is a duplicate for rule {}",
                    ns
                )));
            }
        }
        Ok(RuleSet { rules })
    }

    fn is_valid(&self, s: &str, rule_num: usize) -> AocResult<bool> {
        fn valid<'a>(
            rules: &HashMap<usize, Rule<'_>>,
            s: &'a str,
            rule_num: usize,
        ) -> AocResult<(bool, &'a str)> {
            let rule = rules
                .get(&rule_num)
                .ok_or_else(|| AocError::Process(format!("Rule {} not found", rule_num)))?;
            let mut matched = true;
            let mut remaining = s;
            match rule {
                Rule::Match(ms) => {
                    /*println!(
                        "Rule {}: Checking that '{}' starts with '{}'",
                        rule_num, s, ms
                    );*/
                    if remaining.starts_with(ms) {
                        remaining = &s[ms.len()..];
                    } else {
                        matched = false;
                    }
                }
                Rule::Seq(ov) => {
                    //println!("Rule {}: Checking that '{}' matches {:?}", rule_num, s, ov);
                    for mv in ov.iter() {
                        let mut seq_rem = remaining;
                        matched = true;

                        for nrn in mv.iter() {
                            (matched, seq_rem) = valid(rules, seq_rem, *nrn)?;
                            if !matched {
                                break;
                            }
                        }
                        if matched {
                            remaining = seq_rem;
                            break;
                        }
                    }
                }
            }
            /*println!(
                "Rule {}: String: '{}' Matched: {} Remaining: '{}'",
                rule_num, s, matched, remaining
            );*/
            Ok((matched, remaining))
        }

        // Must have matched the entire string
        let (matched, remaining) = valid(&self.rules, s, rule_num)?;
        if remaining.len() == 0 {
            return Ok(matched);
        }
        Ok(false)
    }
}

#[derive(Debug)]
struct Problem<'a> {
    rule_set: RuleSet<'a>,
    strings: Vec<&'a str>,
}
impl<'a> Problem<'a> {
    fn from_str<P: Part>(s: &'a str) -> AocResult<Self> {
        let secs = s.sections(2)?;
        Ok(Problem {
            rule_set: RuleSet::from_str::<P>(secs[0])?,
            strings: secs[1].lines().collect(),
        })
    }

    fn solve(&self) -> AocResult<u64> {
        let mut sum = 0;
        for s in self.strings.iter() {
            //println!("Checking '{}'", s);
            let valid = self.rule_set.is_valid(s, 0)?;

            if valid {
                sum += 1;
            }
        }
        Ok(sum)
    }
}

pub const SOLUTION: Solution = Solution {
    day: 19,
    name: "Monster Messages",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let problem = Problem::from_str::<PartA>(input)?;

            // Process
            Ok(problem.solve()?)
        },
        // Part b)
        |input| {
            // Generation
            let problem = Problem::from_str::<PartB>(input)?;

            // Process
            Ok(problem.solve()?)
        },
    ],
};
