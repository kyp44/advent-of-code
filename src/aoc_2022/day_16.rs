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
    use aoc::{
        parse::trim,
        tree_search::{GlobalState, GlobalStateTreeNode},
    };
    use derive_new::new;
    use itertools::Itertools;
    use nom::{
        bytes::complete::tag,
        character::complete::alphanumeric1,
        combinator::{map, opt},
        multi::separated_list1,
        sequence::{preceded, tuple},
    };
    use std::{
        collections::{HashMap, HashSet},
        marker::PhantomData,
        ops::Add,
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
            // TODO: Do the reduction up front if raw parsed Valves are not needed for part two.

            // First convert to the final form
            let mut valves = self
                .valves
                .iter()
                .map(|v| {
                    (
                        v.label.as_str(),
                        CondensedValve {
                            label: v.label.as_str(),
                            flow_rate: v.flow_rate,
                            tunnels: v.tunnels.iter().map(|t| Tunnel::from(t.as_str())).collect(),
                        },
                    )
                })
                .collect::<HashMap<_, _>>();

            // Collect those valves with zero flow rate
            let zero_flow_rates = valves
                .values()
                .filter_map(|v| {
                    if v.flow_rate == 0 {
                        Some(v.label)
                    } else {
                        None
                    }
                })
                .collect_vec();

            // Now remove those valves with zero flow rate and substitute tunnels leading there with bypassed tunnels
            for zfr in zero_flow_rates {
                let zfr_tunnels = valves.remove(zfr).unwrap().tunnels;
                let zfr = Tunnel::from(zfr);

                for valve in valves.values_mut() {
                    let valve_tunnels = &mut valve.tunnels;

                    // Remove the ZFR tunnel from the valve tunnels there if it is there
                    if let Some(old_tunnel) = valve_tunnels.take(&zfr) {
                        // Replace with the ZFR tunnels if the path is better than the current path
                        for zfr_tunnel in zfr_tunnels.iter().filter(|t| t.to != valve.label) {
                            let new_tunnel =
                                Tunnel::new(zfr_tunnel.to, old_tunnel.time + zfr_tunnel.time);

                            match valve_tunnels.get(&new_tunnel) {
                                Some(t) => {
                                    if new_tunnel.time < t.time {
                                        valve_tunnels.replace(new_tunnel);
                                    }
                                }
                                None => {
                                    valve_tunnels.insert(new_tunnel);
                                }
                            }
                        }
                    }
                }
            }

            // TODO: Need to deal with the fact that AA, the starting valve, has ZFR, so is currently removed here!

            println!("TODO: {valves:?}");

            // First reduce the valve network to remove valves with no flow rate.

            // TODO: delete
            /* let no_flow_rate_valves = self
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

            fn rec_build_tunnels(valve: &Valve, )

            let condensed_valves = HashMap::new();
            for valve in self.valves.iter().filter(|v| v.flow_rate > 0) {

                let tunnels = valve
                    .tunnels
                    .iter()
                    .map(|t| Tunnel::from(t))
                    .collect_vec();
            } */

            0
        }
    }

    #[derive(Debug, Eq, new)]
    struct Tunnel<'a> {
        to: &'a str,
        time: u8,
    }
    impl PartialEq for Tunnel<'_> {
        fn eq(&self, other: &Self) -> bool {
            self.to == other.to
        }
    }
    impl std::hash::Hash for Tunnel<'_> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.to.hash(state);
        }
    }
    impl<'a> From<&'a str> for Tunnel<'a> {
        fn from(value: &'a str) -> Self {
            Self { to: value, time: 1 }
        }
    }

    // TODO: if raw valves are never needed, refactor this to be the normal Valve
    #[derive(Debug)]
    struct CondensedValve<'a> {
        label: &'a str,
        flow_rate: u8,
        tunnels: HashSet<Tunnel<'a>>,
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
