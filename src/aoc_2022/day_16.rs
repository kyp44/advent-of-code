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
        tree_search::new::{self, GlobalStateAction, GlobalStateTreeNode},
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
    use std::collections::HashSet;

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

    type ValveMap<'a> = IndexMap<&'a str, CondensedValve<'a>>;

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

            // First convert to the final form
            let mut valves: ValveMap<'_> = self
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
                .collect();

            // Verify the starting valve.
            valves
                .get(STARTING_VALVE)
                .ok_or(AocError::Process(
                    format!("Valve {STARTING_VALVE} does not exist!").into(),
                ))
                .and_then(|aa| {
                    if aa.flow_rate > 0 {
                        Err(AocError::Process(
                            format!("The valve {STARTING_VALVE} flow rate is nonzero!").into(),
                        ))
                    } else {
                        Ok(())
                    }
                })?;

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

            // Now remove all valves with zero flow rate and substitute tunnels leading there with bypassed tunnels
            for zfr in zero_flow_rates {
                let zfr_tunnels = if zfr == STARTING_VALVE {
                    // Only keep the starting valve since we need it to start, though no other tunnels will lead here.
                    // We also need the starting valve's tunnels leading to ZFR valves to be replaced.
                    valves.get(zfr).unwrap().tunnels.clone()
                } else {
                    valves.shift_remove(zfr).unwrap().tunnels
                };
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

            let root = ValveNode::initial(&valves);
            let final_state = root.traverse_tree(SearchState::new(&valves));

            for muv in final_state.best_moves.iter() {
                println!("{muv}");
            }

            Ok(final_state.best_cumulative_released)
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
        pub fn open_valve(&mut self, flow_rate: u8) {
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

        // None if time is not up
        pub fn is_time_up(&self) -> Option<u64> {
            (self.time_passed >= MINUTES_ALLOWED).then_some(self.cumulative_released)
        }
    }

    #[derive(Debug, Eq, Clone, new)]
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

    // TODO: Temporary instrumentation for debugging
    #[derive(Debug, Clone)]
    enum Move<'a> {
        TurnOn(&'a str),
        Tunnel(&'a str),
    }
    impl std::fmt::Display for Move<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Move::TurnOn(v) => write!(f, "Turned on valve {}", v),
                Move::Tunnel(to) => write!(f, "Moved through tunnel to {}", to),
            }
        }
    }
    #[derive(Debug, Clone, new)]
    struct MoveRecord<'a> {
        muv: Move<'a>,
        released: u64,
        over_time: u8,
    }
    impl std::fmt::Display for MoveRecord<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{} for {} minutes, releasing {} pressure",
                self.muv, self.over_time, self.released
            )
        }
    }

    #[derive(Debug, new)]
    struct SearchState<'a> {
        valves: &'a ValveMap<'a>,
        #[new(value = "0")]
        best_cumulative_released: u64,
        #[new(value = "Vec::new()")]
        best_moves: Vec<MoveRecord<'a>>,
    }

    #[derive(Debug)]
    struct ValveNode<'a> {
        current: &'a CondensedValve<'a>,
        closed_valves: HashSet<&'a str>,
        pressure_tracker: PressureTracker,
        moves: Vec<MoveRecord<'a>>,
    }
    impl<'a> ValveNode<'a> {
        pub fn initial(valves: &'a ValveMap) -> Self {
            Self {
                current: &valves[STARTING_VALVE],
                closed_valves: valves
                    .keys()
                    .filter(|k| **k != STARTING_VALVE)
                    .copied()
                    .collect(),
                pressure_tracker: PressureTracker::default(),
                moves: Vec::new(),
            }
        }
    }
    impl<'a> GlobalStateTreeNode for ValveNode<'a> {
        type GlobalState = SearchState<'a>;

        fn recurse_action(
            &mut self,
            global_state: &mut Self::GlobalState,
        ) -> GlobalStateAction<Self> {
            // Open this valve if not already open
            let current_label = self.current.label;
            if self.current.flow_rate > 0 && self.closed_valves.contains(current_label) {
                // Opening the valve takes 1 minute (without the valve being open)
                self.pressure_tracker.advance_time(1);

                self.pressure_tracker.open_valve(self.current.flow_rate);
                self.closed_valves.remove(current_label);

                // TODO debugging
                self.moves.push(MoveRecord::new(
                    Move::TurnOn(current_label),
                    self.pressure_tracker.total_flow_per_minute,
                    1,
                ))
            }

            // If all the valves are open, run out the clock and this branch is done
            if self.closed_valves.is_empty() {
                let end_released = self.pressure_tracker.run_out_clock();
                // TODO: Can we utilize a BestMetric here for its handy methods that make this less obnoxious
                if end_released > global_state.best_cumulative_released {
                    println!("TODO terminal {end_released}");
                    global_state.best_cumulative_released = end_released;
                    global_state.best_moves = self.moves.clone();
                }
                return GlobalStateAction::Stop;
            }

            let mut nodes = Vec::with_capacity(self.current.tunnels.len());

            for tunnel in self.current.tunnels.iter() {
                let mut new_pressure_tracker = self.pressure_tracker.clone();

                // Account for tunnel travel time
                new_pressure_tracker.advance_time(tunnel.time);

                // Check if we are out of time
                if let Some(end_released) = new_pressure_tracker.is_time_up() {
                    // TODO: Can we utilize a BestMetric here for its handy methods that make this less obnoxious
                    if end_released > global_state.best_cumulative_released {
                        global_state.best_cumulative_released = end_released;
                        global_state.best_moves = self.moves.clone();
                    }
                    return GlobalStateAction::Stop;
                }

                // TODO debugging
                self.moves.push(MoveRecord::new(
                    Move::Tunnel(tunnel.to),
                    self.pressure_tracker.total_flow_per_minute,
                    tunnel.time,
                ));

                // Go down the tunnels
                nodes.push(Self {
                    current: &global_state.valves[tunnel.to],
                    closed_valves: self.closed_valves.clone(),
                    pressure_tracker: new_pressure_tracker,
                    moves: self.moves.clone(),
                })
            }

            GlobalStateAction::Continue(nodes)
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
