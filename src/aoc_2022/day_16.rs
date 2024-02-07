use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
Valve BB has flow rate=13; tunnels lead to valves CC, AA
Valve CC has flow rate=2; tunnels lead to valves DD, BB
Valve DD has flow rate=20; tunnels lead to valves CC, AA, EE
Valve EE has flow rate=3; tunnels lead to valves FF, DD
Valve FF has flow rate=0; tunnels lead to valves EE, GG
Valve GG has flow rate=0; tunnels lead to valves FF, HH
Valve HH has flow rate=22; tunnel leads to valve GG
Valve II has flow rate=0; tunnels lead to valves AA, JJ
Valve JJ has flow rate=21; tunnel leads to valve II";
            answers = unsigned![1651];
        }
        actual_answers = unsigned![123];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::{
        collections::{HashMap, HashSet},
        marker::PhantomData,
    };

    use aoc::{
        parse::trim,
        tree_search::{GlobalState, GlobalStateTreeNode},
    };
    use derive_new::new;
    use gcollections::ops::IsEmpty;
    use nom::{
        bytes::complete::tag,
        character::complete::alphanumeric1,
        combinator::{map, opt},
        multi::separated_list1,
        sequence::{preceded, tuple},
    };

    use super::*;

    #[derive(Debug)]
    struct Valve {
        label: String,
        flow_rate: u8,
        tunnels: Vec<String>,
    }
    impl Parsable<'_> for Valve {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                tuple((
                    preceded(tag("Valve"), trim(false, alphanumeric1::<&str, _>)),
                    preceded(tag("has flow rate="), nom::character::complete::u8),
                    preceded(
                        tuple((
                            tag("; tunnel"),
                            opt(tag("s")),
                            tag(" lead"),
                            opt(tag("s")),
                            tag(" to valve"),
                            opt(tag("s")),
                        )),
                        trim(false, separated_list1(tag(","), trim(false, alphanumeric1))),
                    ),
                )),
                |(label, flow_rate, tunnels)| Self {
                    label: label.to_string(),
                    flow_rate,
                    tunnels: tunnels.into_iter().map(String::from).collect(),
                },
            )(input)
        }
    }

    #[derive(Debug)]
    pub struct Volcano {
        valves: Vec<Valve>,
    }
    impl FromStr for Volcano {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                valves: Valve::gather(s.lines())?,
            })
        }
    }
    impl Volcano {
        pub fn maximum_pressure_release(&self) -> u64 {
            // First reduce the valve network to remove valves with no flow rate.
            // TODO: Do this up front if raw parsed Valves are not needed for part two.
            let no_flow_rate_valves = self
                .valves
                .iter()
                .filter_map(|v| {
                    if v.flow_rate == 0 {
                        Some((v.label.as_str(), v))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<_, _>>();

            0
        }
    }

    #[derive(new)]
    struct ValveTunnel<'a> {
        to: &'a str,
        time: u8,
    }
    // TODO: if raw valves are never needed, refactor this to be the normal Valve
    struct CondensedValve<'a> {
        flow_rate: u8,
        tunnels: Vec<ValveTunnel<'a>>,
    }

    #[derive(Debug)]
    struct SearchState<'a> {
        valves: HashMap<&'a str, &'a Valve>,
        closed_valves: HashSet<&'a str>,
    }
    impl<'a> GlobalState<ValveNode<'a>> for SearchState<'a> {
        fn update_with_node(&mut self, node: &ValveNode<'a>) {
            todo!()
        }

        fn complete(&self) -> bool {
            todo!()
        }
    }

    #[derive(Debug)]
    struct ValveNode<'a> {
        _dummy: PhantomData<&'a ()>,
    }

    impl<'a> GlobalStateTreeNode for ValveNode<'a> {
        type GlobalState = SearchState<'a>;

        fn recurse_action(
            &self,
            state: &Self::GlobalState,
        ) -> aoc::tree_search::GlobalAction<Self> {
            todo!()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 16,
    name: "Proboscidea Volcanium",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let volcano = Volcano::from_str(input.expect_input()?)?;

            println!("TODO: {}", volcano.maximum_pressure_release());

            // Process
            Ok(0u64.into())
        },
    ],
};
