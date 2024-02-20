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
            answers = unsigned![1651, 1707];
        }
        actual_answers = unsigned![1940, 2469];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{
        parse::trim,
        tree_search::new::{GlobalStateTreeNode, Metric, NodeAction},
    };
    use derive_more::From;
    use derive_new::new;
    use itertools::{iproduct, Itertools};
    use nom::{
        bytes::complete::tag,
        character::complete::alphanumeric1,
        combinator::{map, opt},
        multi::separated_list1,
        sequence::{preceded, tuple},
    };
    use num::rational::Ratio;
    use petgraph::{
        algo::floyd_warshall,
        graph::{DefaultIx, DiGraph, NodeIndex},
        visit::EdgeRef,
    };
    use std::{
        collections::{HashMap, HashSet},
        marker::PhantomData,
    };

    const STARTING_VALVE: &str = "AA";
    const MINUTES_ALLOWED: u8 = 30;
    const ELEPHANT_TEACHING_TIME: u8 = 4;
    const YOU_BEST_TUNNEL_DEPTH: usize = 3;
    const ELEPHANT_BEST_TUNNEL_DEPTH: usize = 2;

    #[derive(Debug)]
    struct ParseValve {
        label: String,
        flow_rate: u8,
        tunnels: Vec<String>,
    }
    impl Parsable<'_> for ParseValve {
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

    #[derive(Debug, Clone, new)]
    struct PressureTracker {
        minutes_allowed: u8,
        #[new(value = "0")]
        cumulative_released: u64,
        #[new(value = "0")]
        total_flow_per_minute: u64,
        #[new(value = "0")]
        time_passed: u8,
    }
    impl PressureTracker {
        // Includes time advance of 1 minute before opening
        pub fn open_valve(&mut self, flow_rate: u8) {
            self.advance_time(1);
            self.total_flow_per_minute += u64::from(flow_rate);
        }

        pub fn advance_time(&mut self, minutes: u8) {
            let time = minutes.min(self.minutes_allowed - self.time_passed);
            self.cumulative_released += self.total_flow_per_minute * u64::from(time);
            self.time_passed += time;
        }

        pub fn run_out_clock(&mut self) -> u64 {
            self.advance_time(self.minutes_allowed);
            self.cumulative_released
        }

        // None if time is not up, otherwise the total pressure released.
        pub fn is_time_up(&self) -> Option<u64> {
            (self.time_passed >= self.minutes_allowed).then_some(self.cumulative_released)
        }
    }

    #[derive(Clone, Copy, From)]
    struct TotalPressure(u64);
    impl Metric for TotalPressure {
        fn is_better(&self, other: &Self) -> bool {
            self.0 > other.0
        }
    }

    #[derive(new)]
    struct SearchState<'a> {
        graph: &'a DiGraph<Valve, u8, DefaultIx>,
        best_tunnel_map: &'a BestTunnelMap,
        #[new(value = "TotalPressure(0)")]
        best_total_pressure: TotalPressure,
    }

    #[derive(Clone)]
    struct Opener {
        pressure_tracker: PressureTracker,
        current_node: NodeIndex,
    }
    impl Opener {
        pub fn new(minutes_allowed: u8, starting_node: NodeIndex) -> Self {
            Self {
                pressure_tracker: PressureTracker::new(minutes_allowed),
                current_node: starting_node,
            }
        }

        pub fn open_current_valve(
            &mut self,
            search_state: &SearchState<'_>,
            closed_valves: &mut HashSet<NodeIndex>,
        ) {
            if closed_valves.contains(&self.current_node) {
                let flow_rate = search_state.graph[self.current_node].flow_rate;

                if flow_rate > 0 {
                    self.pressure_tracker.open_valve(flow_rate);
                }
                closed_valves.remove(&self.current_node);
            }
        }

        pub fn choose_tunnel<'a>(
            &self,
            closed_valves: &HashSet<NodeIndex>,
            best_tunnel_map: &'a BestTunnelMap,
            n: usize,
            exclude: Option<&Tunnel>,
        ) -> Option<&'a Tunnel> {
            best_tunnel_map[&self.current_node]
                .iter()
                .filter(|t| {
                    if let Some(et) = exclude
                        && et.to == t.to
                    {
                        false
                    } else {
                        closed_valves.contains(&t.to)
                    }
                })
                .skip(n)
                .next()
        }

        pub fn travel_tunnel(&self, tunnel: &Tunnel) -> Self {
            let mut pressure_tracker = self.pressure_tracker.clone();
            pressure_tracker.advance_time(tunnel.travel_time);
            Opener {
                pressure_tracker,
                current_node: tunnel.to,
            }
        }
    }

    struct SearchNode<'a> {
        closed_valves: HashSet<NodeIndex>,
        you: Opener,
        elephant: Option<Opener>,
        _phantom: PhantomData<&'a u8>,
    }
    impl SearchNode<'_> {
        pub fn new(
            closed_valves: HashSet<NodeIndex>,
            starting_node: NodeIndex,
            teach_elephant: bool,
        ) -> Self {
            let minutes_allowed = if teach_elephant {
                MINUTES_ALLOWED - ELEPHANT_TEACHING_TIME
            } else {
                MINUTES_ALLOWED
            };

            Self {
                closed_valves,
                you: Opener::new(minutes_allowed, starting_node),
                elephant: teach_elephant.then(|| Opener::new(minutes_allowed, starting_node)),
                _phantom: PhantomData,
            }
        }
    }
    impl<'a> GlobalStateTreeNode for SearchNode<'a> {
        type GlobalState = SearchState<'a>;

        fn recurse_action(mut self, global_state: &mut Self::GlobalState) -> NodeAction<Self> {
            // Open the current valves
            self.you
                .open_current_valve(global_state, &mut self.closed_valves);
            if let Some(el) = self.elephant.as_mut() {
                el.open_current_valve(global_state, &mut self.closed_valves)
            }

            // Have we opened all valves?
            if self.closed_valves.is_empty() {
                let total_pressure = self.you.pressure_tracker.run_out_clock()
                    + self
                        .elephant
                        .as_mut()
                        .map(|o| o.pressure_tracker.run_out_clock())
                        .unwrap_or(0);

                global_state
                    .best_total_pressure
                    .update_if_better(total_pressure.into());
                return NodeAction::Stop;
            }

            // Have we run out of time for either opener?
            if let Some(p) = {
                if let Some(p) = self.you.pressure_tracker.is_time_up() {
                    Some(
                        p + self
                            .elephant
                            .as_mut()
                            .map(|o| o.pressure_tracker.run_out_clock())
                            .unwrap_or(0),
                    )
                } else if let Some(p) = self
                    .elephant
                    .as_mut()
                    .and_then(|o| o.pressure_tracker.is_time_up())
                {
                    Some(p + self.you.pressure_tracker.run_out_clock())
                } else {
                    None
                }
            } {
                global_state.best_total_pressure.update_if_better(p.into());
                return NodeAction::Stop;
            }

            let all_tunnels = iproduct!(0..YOU_BEST_TUNNEL_DEPTH, 0..ELEPHANT_BEST_TUNNEL_DEPTH)
                .filter_map(|(ny, ne)| {
                    let you_tunnel = self.you.choose_tunnel(
                        &self.closed_valves,
                        &global_state.best_tunnel_map,
                        ny,
                        None,
                    );
                    let elephant_tunnel = self.elephant.as_ref().and_then(|e| {
                        e.choose_tunnel(
                            &self.closed_valves,
                            &global_state.best_tunnel_map,
                            ne,
                            you_tunnel,
                        )
                    });

                    (you_tunnel.is_some() || elephant_tunnel.is_some())
                        .then_some((you_tunnel, elephant_tunnel))
                })
                .collect_vec();

            // Branch on the best two tunnels
            NodeAction::Continue(
                all_tunnels
                    .into_iter()
                    .map(|(yt, et)| {
                        let you = match yt {
                            Some(yt) => self.you.travel_tunnel(yt),
                            None => self.you.clone(),
                        };

                        let elephant = self.elephant.as_ref().map(|e| match et {
                            Some(et) => e.travel_tunnel(et),
                            None => e.clone(),
                        });

                        SearchNode {
                            closed_valves: self.closed_valves.clone(),
                            you,
                            elephant,
                            _phantom: PhantomData::default(),
                        }
                    })
                    .collect(),
            )
        }
    }

    #[derive(Debug, new)]
    struct Valve {
        label: String,
        flow_rate: u8,
    }
    impl Valve {
        pub fn open_next_score(&self, time: u8) -> Score {
            Ratio::new(self.flow_rate, time)
        }
    }
    impl std::fmt::Display for Valve {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.label)
        }
    }

    type Score = Ratio<u8>;

    #[derive(Debug)]
    struct Tunnel {
        to: NodeIndex,
        score: Score,
        travel_time: u8,
    }

    type BestTunnelMap = HashMap<NodeIndex, Vec<Tunnel>>;

    #[derive(Debug)]
    pub struct Volcano {
        graph: DiGraph<Valve, u8, DefaultIx>,
        best_tunnel_map: BestTunnelMap,
        starting_node: NodeIndex,
    }
    impl FromStr for Volcano {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let parse_valves = ParseValve::gather(s.lines())?;

            // Build all the graph nodes
            let mut graph = DiGraph::new();
            let node_map = parse_valves
                .iter()
                .map(|valve| {
                    (
                        valve.label.as_str(),
                        graph.add_node(Valve::new(valve.label.to_string(), valve.flow_rate)),
                    )
                })
                .collect::<HashMap<_, _>>();

            // Now build edges
            for valve in parse_valves.iter() {
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

            // Create the table of paths ordered by the best scores.
            let best_tunnel_map: BestTunnelMap = graph
                .node_indices()
                .map(|ni| {
                    (
                        ni,
                        graph
                            .edges(ni)
                            .map(|e| {
                                let to = e.target();
                                let to_valve = &graph[to];
                                let travel_time = *e.weight();
                                Tunnel {
                                    to,
                                    score: to_valve.open_next_score(travel_time),
                                    travel_time,
                                }
                            })
                            .sorted_by(|a, b| a.score.cmp(&b.score).reverse())
                            .collect_vec(),
                    )
                })
                .collect();

            // Validate and get the starting node
            let starting_node = graph
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

            Ok(Self {
                graph,
                best_tunnel_map,
                starting_node,
            })
        }
    }
    impl Volcano {
        pub fn maximum_pressure_released(&self, teach_elephant: bool) -> AocResult<u64> {
            let final_state = SearchNode::new(
                self.graph.node_indices().collect(),
                self.starting_node,
                teach_elephant,
            )
            .traverse_tree(SearchState::new(&self.graph, &self.best_tunnel_map));

            Ok(final_state.best_total_pressure.0)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 16,
    name: "Proboscidea Volcanium",
    preprocessor: Some(|input| Ok(Box::new(Volcano::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Volcano>()?
                .maximum_pressure_released(false)?
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Volcano>()?
                .maximum_pressure_released(true)?
                .into())
        },
    ],
};
