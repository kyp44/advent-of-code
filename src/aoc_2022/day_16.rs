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
        actual_answers = unsigned![1940];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{
        parse::trim,
        tree_search::new::{GlobalStateAction, GlobalStateTreeNode, Metric},
    };
    use derive_more::From;
    use derive_new::new;
    use itertools::Itertools;
    use nom::{
        bytes::complete::tag,
        character::complete::alphanumeric1,
        combinator::{map, opt},
        multi::separated_list1,
        sequence::{preceded, tuple},
    };
    use num::{rational::Ratio, ToPrimitive};
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
        #[new(value = "TotalPressure(0)")]
        best_total_pressure: TotalPressure,
    }

    struct SearchNode<'a> {
        closed_valves: HashSet<NodeIndex>,
        pressure_tracker: PressureTracker,
        current_node: NodeIndex,
        // TODO delete
        number: u8,
        _phantom: PhantomData<&'a u8>,
    }
    impl SearchNode<'_> {
        pub fn new(
            closed_valves: HashSet<NodeIndex>,
            minutes_allowed: u8,
            starting_node: NodeIndex,
        ) -> Self {
            Self {
                closed_valves,
                pressure_tracker: PressureTracker::new(minutes_allowed),
                current_node: starting_node,
                number: 1,
                _phantom: PhantomData,
            }
        }
    }
    impl<'a> GlobalStateTreeNode for SearchNode<'a> {
        type GlobalState = SearchState<'a>;

        fn recurse_action(
            mut self,
            global_state: &mut Self::GlobalState,
        ) -> GlobalStateAction<Self> {
            // Open this valve
            let valve = &global_state.graph[self.current_node];
            if valve.flow_rate > 0 {
                self.pressure_tracker.open_valve(valve.flow_rate);
            }
            self.closed_valves.remove(&self.current_node);

            // Have we opened all valves?
            if self.closed_valves.is_empty() {
                // TODO get rid of x
                let x = self.pressure_tracker.run_out_clock();
                /* global_state
                .best_total_pressure
                .update_if_better(self.pressure_tracker.run_out_clock().into()); */
                global_state.best_total_pressure.update_if_better(x.into());
                println!("TODO Terminal: {x}");
                return GlobalStateAction::Stop;
            }

            // Have we run out of time?
            if let Some(p) = self.pressure_tracker.is_time_up() {
                global_state.best_total_pressure.update_if_better(p.into());
                return GlobalStateAction::Stop;
            }

            #[derive(Debug)]
            struct NextNode {
                node: NodeIndex,
                score: Ratio<u8>,
                travel_time: u8,
            }

            // Determine which to move to next
            let graph = &global_state.graph;
            let mut next_nodes = self
                .closed_valves
                .iter()
                .map(|ni| {
                    let next_valve = &graph[*ni];
                    let travel_time = graph[graph.find_edge(self.current_node, *ni).unwrap()];
                    NextNode {
                        node: *ni,
                        score: next_valve.open_next_score(travel_time),
                        travel_time,
                    }
                })
                .collect_vec();
            next_nodes.sort_by_key(|nn| std::cmp::Reverse(nn.score));

            // Choose the best to can branch for each
            GlobalStateAction::Continue(
                next_nodes
                    .into_iter()
                    .take(2)
                    .map(|nn| {
                        let mut pressure_tracker = self.pressure_tracker.clone();
                        pressure_tracker.advance_time(nn.travel_time);
                        SearchNode {
                            closed_valves: self.closed_valves.clone(),
                            pressure_tracker,
                            current_node: nn.node,
                            number: self.number + 1,
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
        pub fn open_next_score(&self, time: u8) -> Ratio<u8> {
            Ratio::new(self.flow_rate, time)
        }
    }
    impl std::fmt::Display for Valve {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.label)
        }
    }

    #[derive(Debug)]
    pub struct Volcano {
        graph: DiGraph<Valve, u8, DefaultIx>,
        starting_node: NodeIndex,
    }
    impl FromStr for Volcano {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let parse_valves = ParseValve::gather(s.lines())?;

            println!("TODO building graph...");

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

            for node_index in graph.node_indices() {
                let valve = &graph[node_index];
                println!("Surviving node: {node_index:?}, {}", valve.label);
            }

            // TODO save out graph for viewing
            println!("{}", petgraph::dot::Dot::new(&graph));

            println!("TODO done!");

            // Print out valves and paths in descending order of score
            struct NodePaths<'a> {
                node: &'a str,
                paths: Vec<NodePath<'a>>,
            }
            impl std::fmt::Display for NodePaths<'_> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(
                        f,
                        "{} edges: {}",
                        self.node,
                        self.paths.iter().map(|p| format!("{p}")).join(", ")
                    )
                }
            }
            struct NodePath<'a> {
                node: &'a str,
                score: Ratio<u8>,
            }
            impl std::fmt::Display for NodePath<'_> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}: {}", self.node, self.score.to_f32().unwrap())
                }
            }
            graph
                .node_indices()
                .map(|ni| NodePaths {
                    node: &graph[ni].label,
                    paths: graph
                        .edges(ni)
                        .map(|e| {
                            let new_valve = &graph[e.target()];
                            NodePath {
                                node: &new_valve.label,
                                score: new_valve.open_next_score(*e.weight()),
                            }
                        })
                        .sorted_by(|a, b| a.score.cmp(&b.score).reverse())
                        .collect(),
                })
                .for_each(|np| println!("{np}"));

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
                starting_node,
            })
        }
    }
    impl Volcano {
        pub fn maximum_pressure_released(&self, teach_elephant: bool) -> AocResult<u64> {
            let final_state = SearchNode::new(
                self.graph.node_indices().collect(),
                MINUTES_ALLOWED,
                self.starting_node,
            )
            .traverse_tree(SearchState::new(&self.graph));

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
    ],
};
