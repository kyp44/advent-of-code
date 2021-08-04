use crate::aoc::prelude::*;
use crate::aoc::trim;
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{digit1, space1},
    combinator::map,
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, separated_pair},
    Finish,
};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Number;

    solution_test! {
    vec![Number(149), Number(332)],
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
    vec![Some(Number(2)), None],
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
    vec![3, 12].answer_vec()
    }
}

#[derive(Debug)]
enum Rule<'a> {
    Match(&'a str),
    Seq(Vec<Vec<usize>>),
}
impl<'a> Parseable<'a> for Rule<'a> {
    fn parser(input: &'a str) -> NomParseResult<Self> {
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
    fn fix_rule<'a>(&self, _num: usize, rule: Rule<'a>) -> Rule<'a> {
        rule
    }
}
struct PartA;
impl Part for PartA {}
struct PartB;
impl Part for PartB {
    fn fix_rule<'a>(&self, num: usize, rule: Rule<'a>) -> Rule<'a> {
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
    fn from_str(s: &'a str, part: &dyn Part) -> AocResult<Self> {
        let mut rules = HashMap::new();
        for line in s.lines() {
            let (ns, rule) = separated_pair(digit1, tag(":"), Rule::parser)(line)
                .finish()
                .discard_input()?;
            let n: usize = ns.parse().unwrap();
            if rules.insert(n, part.fix_rule(n, rule)).is_some() {
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
            rule_set: &RuleSet,
            s: &'a str,
            rule_num: usize,
            level: usize,
        ) -> AocResult<(bool, &'a str)> {
            let _tab: String = (0..level).map(|_| "  ").collect();
            let rule = rule_set
                .rules
                .get(&rule_num)
                .ok_or_else(|| AocError::Process(format!("Rule {} not found", rule_num)))?;
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

    fn _rule_string(&self, rule_num: &usize) -> String {
        let rule = self.rules.get(rule_num).unwrap();

        match rule {
            Rule::Match(s) => s.to_string(),
            Rule::Seq(ov) => {
                format!(
                    "({})",
                    ov.iter()
                        .map(|sv| sv
                            .iter()
                            .map(|rn| self._rule_string(rn))
                            .collect::<String>())
                        .join(" | ")
                )
            }
        }
    }
}

#[derive(Debug)]
struct Problem<'a> {
    rule_set: RuleSet<'a>,
    strings: Vec<&'a str>,
}
impl<'a> Problem<'a> {
    fn from_str(s: &'a str, part: &dyn Part) -> AocResult<Self> {
        let secs = s.sections(2)?;
        Ok(Problem {
            rule_set: RuleSet::from_str(secs[0], part)?,
            strings: secs[1].lines().collect(),
        })
    }

    fn solve(&self) -> AocResult<Answer> {
        let mut sum = 0;

        //println!("Valid strings:");
        for s in self.strings.iter() {
            //println!("\nChecking '{}' for rule {}", s, rule_num);
            //println!("{}", self.rule_set._rule_string(rule_num));
            if self.rule_set.is_valid(s, 0)? {
                //println!("{}", s);
                sum += 1;
            }
        }

        Ok(sum.into())
    }
}

pub const SOLUTION: Solution = Solution {
    day: 19,
    name: "Monster Messages",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let problem = Problem::from_str(input, &PartA)?;

            // Process
            problem.solve()
        },
        // Part b)
        |input| {
            // Generation
            let problem = Problem::from_str(input, &PartB)?;

            // Process
            problem.solve()
        },
    ],
};
