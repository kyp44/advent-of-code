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
    use super::*;
    use aoc::{
        parse::trim,
        tree_search::new::{GlobalStateAction, GlobalStateTreeNode},
    };
    use derive_new::new;
    use indexmap::IndexMap;
    use itertools::Itertools;
    use nom::{
        bytes::complete::tag,
        character::complete::alphanumeric1,
        combinator::{map, opt},
        multi::separated_list1,
        sequence::{preceded, tuple},
    };
    use num::rational::Ratio;
    use petgraph::{algo::floyd_warshall, graph::DiGraph, visit::EdgeRef};
    use std::collections::{HashMap, HashSet};

    const STARTING_VALVE: &str = "AA";
    const MINUTES_ALLOWED: u8 = 30;

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
        pub fn maximum_pressure_release(&self) -> AocResult<u64> {
            // TODO: Do the reduction up front if raw parsed Valves are not needed for part two.

            println!("TODO building graph...");

            // Build all the graph nodes
            let mut graph = DiGraph::new();
            let node_map = self
                .valves
                .iter()
                .map(|valve| {
                    (
                        valve.label.as_str(),
                        graph.add_node(ValveNode::new(&valve.label, valve.flow_rate)),
                    )
                })
                .collect::<HashMap<_, _>>();

            // Now build edges
            for valve in self.valves.iter() {
                for tunnel in valve.tunnels.iter() {
                    graph.add_edge(
                        node_map[valve.label.as_str()],
                        node_map[tunnel.as_str()],
                        1u8,
                    );
                }
            }

            // Now determine shortest paths between all pairs
            let shortest_paths = floyd_warshall(&graph, |e| *e.weight()).unwrap();

            // Add shortest path edges
            for ((a, b), time) in shortest_paths.into_iter() {
                if a != b {
                    graph.update_edge(a, b, time);
                }
            }

            // Now remove all zero flow rate nodes except the starting node
            graph.retain_nodes(|graph, ni| {
                let valve = &graph[ni];
                valve.flow_rate > 0 || valve.label == STARTING_VALVE
            });

            for node_index in graph.node_indices() {
                let valve = &graph[node_index];
                println!("Surviving node: {node_index:?}, {}", valve.label);
            }

            // TODO save out graph for viewing
            println!("{}", petgraph::dot::Dot::new(&graph));

            println!("TODO done!");

            // TODO test code
            let valves = graph
                .node_indices()
                .filter(|ni| graph[*ni].label != STARTING_VALVE)
                .collect_vec();

            println!("TODO total nodes: {}", graph.node_count());
            println!("TODO numb of nodes to permute: {}", valves.len());

            for ni in graph.node_indices() {
                println!(
                    "TODO {} edges FR/T: {}",
                    graph[ni].label,
                    graph
                        .edges(ni)
                        .map(|e| {
                            let target = &graph[e.target()];
                            let t = *e.weight();
                            let fr = target.flow_rate;
                            format!(
                                "{}: {}/{}-{}",
                                target.label,
                                fr,
                                t,
                                target.open_next_score(t),
                            )
                        })
                        .join(", ")
                );
            }

            // First try naive algorithm
            let mut closed_valves = graph
                .node_indices()
                .filter(|ni| graph[*ni].flow_rate > 0)
                .collect::<HashSet<_>>();
            let mut pressure_tracker = PressureTracker::default();
            let mut current_node = graph
                .node_indices()
                .filter_map(|ni| {
                    let valve = &graph[ni];

                    if valve.label == STARTING_VALVE {
                        Some(if valve.flow_rate != 0 {
                            Err(AocError::Process(
                                format!("The valve {STARTING_VALVE} flow rate is nonzero!").into(),
                            ))
                        } else {
                            Ok(ni)
                        })
                    } else {
                        None
                    }
                })
                .next()
                .ok_or(AocError::Process(
                    format!("Valve {STARTING_VALVE} does not exist!").into(),
                ))??;

            let x = loop {
                // Have we opened all valves?
                if closed_valves.is_empty() {
                    break pressure_tracker.run_out_clock();
                }

                // Have we run out of time?
                if let Some(p) = pressure_tracker.is_time_up() {
                    break p;
                }

                // Determine which to move to next
                let (next_node, next_valve, time) = closed_valves
                    .iter()
                    .map(|ni| {
                        let valve = &graph[*ni];
                        let time = graph[graph.find_edge(current_node, *ni).unwrap()];
                        (*ni, valve, time)
                    })
                    .max_by_key(|(_, v, t)| v.open_next_score(*t))
                    .unwrap();

                // Move to the next valve
                pressure_tracker.advance_time(time);

                // Open the next valve
                pressure_tracker.open_valve(next_valve.flow_rate);
                closed_valves.remove(&next_node);

                current_node = next_node;
            };

            println!("TODO Best: {x}");

            Ok(0)
        }
    }

    #[derive(new)]
    struct ValveNode<'a> {
        // TODO: in the end we may not need the label at and can just use the flow rate as the weight.
        label: &'a str,
        flow_rate: u8,
    }
    impl ValveNode<'_> {
        pub fn open_next_score(&self, time: u8) -> Ratio<u8> {
            Ratio::new(self.flow_rate, time)
        }
    }
    impl std::fmt::Display for ValveNode<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.label)
        }
    }

    #[derive(Debug, Clone)]
    struct PressureTracker {
        cumulative_released: u64,
        total_flow_per_minute: u64,
        time_passed: u8,
    }
    impl Default for PressureTracker {
        fn default() -> Self {
            Self {
                cumulative_released: 0,
                total_flow_per_minute: 0,
                time_passed: 0,
            }
        }
    }
    impl PressureTracker {
        // Includes time advance of 1 minute before opening
        pub fn open_valve(&mut self, flow_rate: u8) {
            self.advance_time(1);
            self.total_flow_per_minute += u64::from(flow_rate);
        }

        pub fn advance_time(&mut self, minutes: u8) {
            let time = minutes.min(MINUTES_ALLOWED - self.time_passed);
            self.cumulative_released += self.total_flow_per_minute * u64::from(time);
            self.time_passed += time;
        }

        pub fn run_out_clock(&mut self) -> u64 {
            self.advance_time(MINUTES_ALLOWED);
            self.cumulative_released
        }

        // None if time is not up, otherwise the total pressure released.
        pub fn is_time_up(&self) -> Option<u64> {
            (self.time_passed >= MINUTES_ALLOWED).then_some(self.cumulative_released)
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

            println!("TODO: {}", volcano.maximum_pressure_release()?);

            // Process
            Ok(0u64.into())
        },
    ],
};
