use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    const EXAMPLE_INPUT: &str = "Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.
Blueprint 2: Each ore robot costs 2 ore. Each clay robot costs 3 ore. Each obsidian robot costs 3 ore and 8 clay. Each geode robot costs 3 ore and 12 obsidian.";

    solution_tests! {
        example {
            input = EXAMPLE_INPUT;
            answers = unsigned![33];
        }
        expensive_example {
            input = EXAMPLE_INPUT;
            answers = &[None, Some(Answer::Unsigned(3472))];
        }
        actual_answers = unsigned![1294, 13640];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{
        parse::trim,
        tree_search::{GlobalStateTreeNode, Metric, NodeAction},
    };
    use derive_more::{Add, Deref, DerefMut, From};
    use derive_new::new;
    use infinitable::Infinitable;
    use itertools::Itertools;
    use maplit::hashmap;
    use multiset::HashMultiSet;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::space1,
        combinator::map,
        multi::{many_m_n, separated_list1},
        sequence::{delimited, pair, separated_pair, terminated},
    };
    use num::rational::Ratio;
    use std::{collections::HashMap, hash::Hash};
    use strum::IntoEnumIterator;
    use strum_macros::EnumIter;

    /// The number of different robots that can be built.
    const NUM_ROBOTS: usize = 4;

    /// A type of material or robot that mines the material.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
    enum Material {
        /// Ore.
        Ore,
        /// Clay.
        Clay,
        /// Obsidian.
        Obsidian,
        /// A geode.
        Geode,
    }
    impl Parsable<'_> for Material {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            alt((
                map(tag("ore"), |_| Self::Ore),
                map(tag("clay"), |_| Self::Clay),
                map(tag("obsidian"), |_| Self::Obsidian),
                map(tag("geode"), |_| Self::Geode),
            ))(input)
        }
    }

    /// A build cost for for only one material, which can be parsed from text input.
    #[derive(Debug)]
    struct ParseCost {
        /// The material.
        material: Material,
        /// The number of the material required to build.
        cost: u8,
    }
    impl Parsable<'_> for ParseCost {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(nom::character::complete::u8, space1, Material::parser),
                |(cost, material)| Self { material, cost },
            )(input)
        }
    }

    /// A complete cost to build a robot, which can be parsed from text input.
    #[derive(Debug)]
    struct ParseRobotCost {
        /// The type of robot to be build.
        robot_type: Material,
        /// The total cost of all materials to required to build the robot.
        cost: RobotCost,
    }
    impl Parsable<'_> for ParseRobotCost {
        fn parser(input: &'_ str) -> NomParseResult<&str, Self> {
            map(
                pair(
                    delimited(
                        trim(false, tag("Each")),
                        Material::parser,
                        trim(false, tag("robot costs")),
                    ),
                    terminated(RobotCost::parser, trim(false, tag("."))),
                ),
                |(robot, cost)| Self {
                    robot_type: robot,
                    cost,
                },
            )(input)
        }
    }

    /// The costs for every type of robot.
    ///
    /// The key is the robot type, and the value is the total cost to build that
    /// robot.
    type RobotCosts = HashMap<Material, RobotCost>;

    /// A blueprint of robot build costs, which can be parsed from text input.
    #[derive(Debug)]
    struct Blueprint {
        /// The blueprint number.
        num: u8,
        /// The costs to build each type of robot for this blueprint.
        robot_costs: RobotCosts,
    }
    impl Parsable<'_> for Blueprint {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                pair(
                    delimited(
                        trim(false, tag("Blueprint")),
                        nom::character::complete::u8,
                        trim(false, tag(":")),
                    ),
                    many_m_n(NUM_ROBOTS, NUM_ROBOTS, ParseRobotCost::parser),
                ),
                |(num, costs)| Self {
                    num,
                    robot_costs: costs
                        .into_iter()
                        .map(|prc| (prc.robot_type, prc.cost))
                        .collect(),
                },
            )(input)
        }
    }
    impl Blueprint {
        /// Determines and returns the largest number of geodes that can possibly be cracked open
        /// using this blueprint when `time_allowed` minutes are allowed for building robots.
        pub fn largest_geodes_cracked(&self, time_allowed: usize) -> u64 {
            let search_state = SearchNode::new(self, time_allowed)
                .traverse_tree(SearchState::new(self, time_allowed));

            search_state.most_geodes_cracked.0.try_into().unwrap()
        }
    }

    /// The global state for the tree search of possible build recipes.
    struct SearchState {
        /// The constant overall time allowed in minutes.
        time_allowed: usize,
        /// The largest number of geodes that have been cracked so far for any recipe.
        most_geodes_cracked: GeodesCracked,
        /// The maximum number of robots that we would ever need to support building
        /// each robot type in a single minute.
        max_robots_needed: RobotInventory,
        /// For each robot type, the maximum number of minutes after which we would no
        /// longer want to build that type of robot as doing so would not be able to
        /// impact the number of geodes cracked by the time we must stop.
        max_build_time: HashMap<Material, usize>,
    }
    impl SearchState {
        /// Creates a new search state for a particular `blueprint` and `time_allowed` in minutes.
        pub fn new(blueprint: &Blueprint, time_allowed: usize) -> Self {
            let mut max_robots_needed = RobotInventory::default();
            for rt in Material::iter() {
                let max_cost = blueprint
                    .robot_costs
                    .values()
                    .map(|c| c.count_of(&rt))
                    .max()
                    .unwrap();

                max_robots_needed
                    .insert_times(rt, if max_cost > 0 { max_cost } else { time_allowed })
            }

            let max_value = |t| (Ratio::new(t, 24) * time_allowed).ceil().to_integer();
            let max_build_time = hashmap! [
                Material::Ore => max_value(14),
                Material::Clay => max_value(16),
                Material::Obsidian => max_value(20),
                Material::Geode => max_value(22),
            ];

            Self {
                time_allowed,
                most_geodes_cracked: GeodesCracked(0),
                max_robots_needed,
                max_build_time,
            }
        }
    }

    /// The next robot to build.
    #[derive(Clone, new)]
    struct ToBuildNext {
        /// The time, in minutes, needed to wait before the robot can be built.
        time_to_build: usize,
        /// The type of robot to be built.
        to_build: Material,
    }

    /// A node for the build recipe tree search.
    ///
    /// This can represent one or more minutes of time.
    #[derive(Clone)]
    struct SearchNode<'a> {
        /// The current state of the time tracker.
        time_tracker: TimeTracker<'a>,
        /// The next robot to build during this period of time, if any.
        to_build_next: Option<ToBuildNext>,
    }
    impl<'a> SearchNode<'a> {
        /// Creates a new search node for a `blueprint` and `time_allowed` in minutes.
        ///
        /// This is initialized at zero time.
        fn new(blueprint: &'a Blueprint, time_allowed: usize) -> Self {
            Self {
                time_tracker: TimeTracker::new(&blueprint.robot_costs, time_allowed),
                to_build_next: None,
            }
        }

        /// Duplicates this node to create a child that will wait to build the robot
        /// specified in `to_build_next`.
        fn duplicate(&self, to_build_next: ToBuildNext) -> Self {
            Self {
                time_tracker: self.time_tracker.clone(),
                to_build_next: Some(to_build_next),
            }
        }
    }
    impl GlobalStateTreeNode for SearchNode<'_> {
        type GlobalState = SearchState;

        fn recurse_action(mut self, global_state: &mut Self::GlobalState) -> NodeAction<Self> {
            // Wait to build our current robot
            if let Some(ref ttb) = self.to_build_next {
                if let Err(gc) = self
                    .time_tracker
                    .wait_to_build_robot(ttb)
                    .and_then(|_| self.time_tracker.time_up())
                {
                    global_state.most_geodes_cracked.update_if_better(gc);
                    return NodeAction::Stop;
                }
            }

            // See which robots we can and should build next
            let children = Material::iter()
                .filter_map(|to_build| {
                    if let Infinitable::Finite(t) = self.time_tracker.time_to_build_robot(&to_build)
                    // Do not build anything after the time is up.
                    && self.time_tracker.elapsed_time + t < global_state.time_allowed
                    // There is an upper limit on how many robots we need of a given type.
                    && self.time_tracker.robots.count_of(&to_build)
                        < global_state.max_robots_needed.count_of(&to_build)
                    // Beyond a certain time, we should not be building a robot of a given type.
                    && self.time_tracker.elapsed_time + t <= global_state.max_build_time[&to_build]
                    {
                        Some(self.duplicate(ToBuildNext::new(t, to_build)))
                    } else {
                        None
                    }
                })
                .collect_vec();

            // If we have nothing to build, then we can just run out the clock
            if children.is_empty() {
                global_state
                    .most_geodes_cracked
                    .update_if_better(self.time_tracker.run_out_clock());
                NodeAction::Stop
            } else {
                NodeAction::Continue(children)
            }
        }
    }

    /// A new type for the number of geodes cracked open.
    #[derive(Default, Clone, Copy, Add, From)]
    struct GeodesCracked(usize);
    impl Metric for GeodesCracked {
        fn is_better(&self, other: &Self) -> bool {
            self.0 > other.0
        }
    }

    /// A new type for multi sets.
    ///
    /// This is just a wrapper around [`HashMultiSet`] that adds some additional
    /// methods and trait implementations.
    /// This can be parsed from text input for a [`RobotCost`].
    #[derive(Deref, DerefMut, Clone)]
    struct MultiSet<K: Eq + Hash>(HashMultiSet<K>);
    impl<K: Eq + Hash> Default for MultiSet<K> {
        fn default() -> Self {
            Self(HashMultiSet::new())
        }
    }
    impl<K: Eq + Hash + std::fmt::Debug> std::fmt::Debug for MultiSet<K> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                self.distinct_elements()
                    .map(|k| format!("{k:?}: {}", self.count_of(k)))
                    .join(" ")
            )
        }
    }
    impl From<Vec<ParseCost>> for MultiSet<Material> {
        fn from(value: Vec<ParseCost>) -> Self {
            let mut cost_set = HashMultiSet::new();

            for cost in value {
                cost_set.insert_times(cost.material, cost.cost.into());
            }

            Self(cost_set)
        }
    }
    impl<K: Eq + Hash> MultiSet<K> {
        /// Returns the count for the element with the largest multiplicity.
        pub fn max_count(&self) -> usize {
            self.distinct_elements()
                .map(|k| self.count_of(k))
                .max()
                .unwrap_or(0)
        }
    }
    impl<K: Eq + Hash> std::ops::SubAssign<&Self> for MultiSet<K> {
        fn sub_assign(&mut self, rhs: &Self) {
            for key in rhs.distinct_elements() {
                self.remove_times(key, rhs.count_of(key));
            }
        }
    }
    impl Parsable<'_> for RobotCost {
        fn parser(input: &'_ str) -> NomParseResult<&str, Self> {
            map(
                separated_list1(trim(false, tag("and")), ParseCost::parser),
                |costs| costs.into(),
            )(input)
        }
    }

    /// A total cost to build a robot, that is the number of each material required to
    /// build it.
    type RobotCost = MultiSet<Material>;
    /// Inventory of materials that have been harvested.
    type MaterialInventory = MultiSet<Material>;
    /// Inventory of robots that have been built.
    type RobotInventory = MultiSet<Material>;

    /// Tracks the passage of time, manages material and robot inventories, and handles the
    /// harvesting of materials.
    #[derive(Debug, Clone)]
    struct TimeTracker<'a> {
        /// The cost of each type of robot.
        robot_costs: &'a RobotCosts,
        /// The total time allowed in minutes.
        time_allowed: usize,
        /// The current material inventory.
        materials: MaterialInventory,
        /// The current robot inventory.
        robots: RobotInventory,
        /// The elapsed time so far in minutes.
        elapsed_time: usize,
    }
    impl<'a> TimeTracker<'a> {
        /// Creates a new time tracker at time zero `robot_costs` and total
        /// `time_allowed` in minutes.
        pub fn new(robot_costs: &'a RobotCosts, time_allowed: usize) -> Self {
            // We start with one ore-collecting robot
            let mut robots = RobotInventory::default();
            robots.insert(Material::Ore);

            Self {
                robot_costs,
                time_allowed,
                materials: MaterialInventory::default(),
                robots,
                elapsed_time: 0,
            }
        }

        /// Returns the number of geodes cracked open so far.
        pub fn geodes_cracked(&self) -> GeodesCracked {
            self.materials.count_of(&Material::Geode).into()
        }

        /// Passes one minute of time, harvesting materials using the
        /// current inventory of robots.
        ///
        /// Does nothing if time is up and returns the final number
        /// of geodes cracked open.
        /// Otherwise returns nothing.
        fn tick(&mut self) -> Result<(), GeodesCracked> {
            self.time_up()?;

            // Have the robots collect materials
            for robot in self.robots.iter() {
                self.materials.insert(*robot)
            }

            // Increment time
            self.elapsed_time += 1;

            Ok(())
        }

        /// Returns the final number of geodes cracked open if time is up,
        /// otherwise returns nothing.
        pub fn time_up(&self) -> Result<(), GeodesCracked> {
            if self.elapsed_time >= self.time_allowed {
                Err(self.geodes_cracked())
            } else {
                Ok(())
            }
        }

        /// Tests whether or not a robot can be built with the current material
        /// inventory.
        pub fn can_build_robot(&self, to_build: &Material) -> bool {
            let robot_cost = &self.robot_costs[to_build];
            robot_cost
                .distinct_elements()
                .all(|m| self.materials.count_of(m) >= robot_cost.count_of(m))
        }

        /// Spends materials to build a robot.
        ///
        /// Note that no checks are done to ensure that the material inventory
        /// is sufficient, hence the robot will *always* be built.
        fn build_robot(&mut self, robot_type: Material) {
            // Spend the materials
            self.materials -= &self.robot_costs[&robot_type];

            // Add the new robot
            self.robots.insert(robot_type);
        }

        /// Advances time by one minute, optionally building a robot during the minute.
        ///
        /// This will panic if we cannot afford to build the robot.
        /// Does nothing if time is up and returns the final number
        /// of geodes cracked open.
        /// Otherwise returns nothing.
        pub fn advance_time(&mut self, to_build: Option<Material>) -> Result<(), GeodesCracked> {
            if let Some(rt) = to_build
                && !self.can_build_robot(&rt)
            {
                panic!("cannot afford to build a {rt:?} robot!");
            }

            self.tick()?;

            if let Some(rt) = to_build {
                self.build_robot(rt)
            }

            Ok(())
        }

        /// Waits the required time specified in `to_build_next` then builds
        /// the specified robot.
        ///
        /// If time runs out during this time, nothing further is done, and
        /// the final number of geodes cracked open is returned.
        /// Otherwise returns nothing.
        pub fn wait_to_build_robot(
            &mut self,
            to_build_next: &ToBuildNext,
        ) -> Result<(), GeodesCracked> {
            for _ in 0..to_build_next.time_to_build {
                self.advance_time(None)?;
            }
            self.advance_time(Some(to_build_next.to_build))
        }

        /// Runs out the clock, building no more robots.
        ///
        /// Returns the final number of geodes cracked.
        pub fn run_out_clock(&mut self) -> GeodesCracked {
            loop {
                if let Err(gc) = self.tick() {
                    break gc;
                }
            }
        }

        /// Calculates the time left in minutes until a given robot can be built,
        /// taking into account the materials that will be gathered during the time
        /// period.
        ///
        /// This assumes that no other robots would be built during the time.
        /// If the robot cannot be built at all given the currently available robots,
        /// then [`Infinitable::Infinity`] is returned.
        pub fn time_to_build_robot(&self, to_build: &Material) -> Infinitable<usize> {
            let mut times = MultiSet::default();

            for material in Material::iter() {
                let cost = self.robot_costs[to_build].count_of(&material);
                let num_robots = self.robots.count_of(&material);
                if cost > 0 {
                    if num_robots == 0 {
                        return Infinitable::Infinity;
                    } else {
                        times.insert_times(
                            material,
                            cost.saturating_sub(self.materials.count_of(&material))
                                .div_ceil(num_robots),
                        )
                    }
                }
            }

            Infinitable::Finite(times.max_count())
        }
    }

    /// The overall robot factory, which can be parsed from text input.
    #[derive(Debug)]
    pub struct RobotFactory {
        /// The parsed set of blueprints.
        blueprints: Vec<Blueprint>,
    }
    impl FromStr for RobotFactory {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                blueprints: Blueprint::gather(s.lines())?,
            })
        }
    }
    impl RobotFactory {
        /// Returns the sum of quality levels after determining the most geodes that can be cracked using
        /// each blueprint (part one).
        ///
        /// Here 24 minutes are allowed in total.
        pub fn sum_of_quality_levels(&self) -> u64 {
            self.blueprints
                .iter()
                .map(|b| u64::from(b.num) * b.largest_geodes_cracked(24))
                .sum()
        }

        /// Returns the product of the most geodes that can be cracked for each of the first three
        /// blueprints (part two).
        ///
        /// Here 32 minutes are allowed in total.
        pub fn product_of_most_geodes(&self) -> u64 {
            self.blueprints
                .iter()
                .take(3)
                .map(|b| b.largest_geodes_cracked(32))
                .product()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 19,
    name: "Not Enough Minerals",
    preprocessor: Some(|input| Ok(Box::new(RobotFactory::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<RobotFactory>()?
                .sum_of_quality_levels()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<RobotFactory>()?
                .product_of_most_geodes()
                .into())
        },
    ],
};
