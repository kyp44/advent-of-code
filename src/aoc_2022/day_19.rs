use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
                example {
                    input = "Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.
        Blueprint 2: Each ore robot costs 2 ore. Each clay robot costs 3 ore. Each obsidian robot costs 3 ore and 8 clay. Each geode robot costs 3 ore and 12 obsidian.";
                    answers = unsigned![33, 3472];
                }
                // TODO
                /* example {
                    input = "Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.
    Blueprint 2: Each ore robot costs 2 ore. Each clay robot costs 3 ore. Each obsidian robot costs 3 ore and 8 clay. Each geode robot costs 3 ore and 12 obsidian.
    Blueprint 3: Each ore robot costs 4 ore. Each clay robot costs 4 ore. Each obsidian robot costs 3 ore and 11 clay. Each geode robot costs 3 ore and 8 obsidian.
    Blueprint 4: Each ore robot costs 3 ore. Each clay robot costs 4 ore. Each obsidian robot costs 4 ore and 5 clay. Each geode robot costs 3 ore and 12 obsidian.";
                    answers = unsigned![71];
                } */
                actual_answers = unsigned![1294];
            }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{
        parse::trim,
        tree_search::{GlobalStateTreeNode, Metric, NodeAction},
    };
    use derive_more::{Add, Deref, DerefMut};
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
    use std::{
        cmp::Ordering,
        collections::{hash_map, HashMap},
        hash::Hash,
    };
    use strum::IntoEnumIterator;
    use strum_macros::EnumIter;

    const NUM_ROBOTS: usize = 4;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
    enum Material {
        Ore,
        Clay,
        Obsidian,
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

    #[derive(Debug)]
    struct ParseCost {
        material: Material,
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

    #[derive(Debug)]
    struct ParseRobotCost {
        robot_type: Material,
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

    type RobotCosts = HashMap<Material, RobotCost>;

    #[derive(Debug)]
    struct Blueprint {
        num: u8,
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
        pub fn largest_geodes_cracked(&self, time_allowed: usize) -> u64 {
            let search_state = SearchNode::new(self, time_allowed)
                .traverse_tree(SearchState::new(self, time_allowed));

            // TODO: print recipe
            println!("Recipe:",);
            for (min, built) in search_state.build_recipe.into_iter().enumerate() {
                println!("{min}. {built:?}");
            }

            return search_state.most_geodes_cracked.0.try_into().unwrap();

            // TODO test code
            let mut time_tracker = TimeTracker::new(&self.robot_costs, time_allowed);

            for build in BUILD_RECIPES[usize::from(self.num) - 1] {
                // TODO now calculate the number of turns to build each robot
                /* for robot_type in Material::iter() {
                    let time_to_build = time_tracker.time_to_build_robot(&robot_type);
                    println!(
                        "Time to build {:?}: {:?}",
                        robot_type,
                        annoying(&time_to_build),
                    );
                }
                println!(); */

                /* println!("Goal: {goal:?}");
                let time_to_build_goal = annoying(&time_tracker.time_to_build_robot(&goal));
                println!("Time to build goal: {:?}", time_to_build_goal);
                if !time_to_build_goal.is_finite() {
                    return Err(AocError::NoSolution);
                }
                let time_to_build_goal = time_to_build_goal.finite().unwrap();
                if time_to_build_goal == 0 {
                    // We can build our goal now!
                    println!("We should definitely build a {goal:?} robot!");
                    goal = goals.next().unwrap_or(goal);
                } else {
                    // Can we build any other robot that would help?
                    if let Some((build, time)) = Material::iter()
                        .filter_map(|rt| {
                            if time_tracker.can_build_robot(&rt) {
                                let mut future = time_tracker.clone();
                                asdfasdgd Need to fix this and use advance_time
                                future.tick();
                                future.build_robot(rt);

                                let new_goal_time = annoying(&future.time_to_build_robot(&goal))
                                    .finite()
                                    .unwrap()
                                    + 1;

                                (new_goal_time <= time_to_build_goal).then_some((rt, new_goal_time))
                            } else {
                                None
                            }
                        })
                        .min_by_key(|t| t.1)
                    {
                        println!("We should build a {build:?} robot, as this will reduce our goal time to build to {time}");
                    }
                } */

                // Check whether we should build
                let prob = if let Some(to_build) = build {
                    let time_to_build = time_tracker
                        .time_to_build_robot(&to_build)
                        .finite()
                        .unwrap();
                    if !time_tracker.should_build_robot(time_to_build, to_build) {
                        println!("At minute {} we want to build a {to_build:?} robot, but really shouldn't!", time_tracker.elapsed_time + 1);
                        println!(
                            "Before build: {:?}",
                            time_tracker.time_to_build_important(&to_build)
                        );
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                let _ = time_tracker.advance_time(build);

                if prob {
                    println!(
                        "After build: {:?}\n",
                        time_tracker.time_to_build_important(&build.unwrap())
                    );
                }
            }

            time_tracker.geodes_cracked().try_into().unwrap()
        }
    }

    const BUILD_RECIPES: &[[Option<Material>; 24]] = &[
        // Blueprint 1
        [
            None,
            None,
            Some(Material::Clay),
            None,
            Some(Material::Clay),
            None,
            Some(Material::Clay),
            None,
            None,
            None,
            Some(Material::Obsidian),
            Some(Material::Clay),
            None,
            None,
            Some(Material::Obsidian),
            None,
            None,
            Some(Material::Geode),
            None,
            None,
            Some(Material::Geode),
            None,
            None,
            None,
        ],
        // Blueprint 2
        [
            None,
            None,
            Some(Material::Ore),
            None,
            Some(Material::Ore),
            Some(Material::Clay),
            Some(Material::Clay),
            Some(Material::Clay),
            Some(Material::Clay),
            Some(Material::Clay),
            Some(Material::Obsidian),
            None,
            Some(Material::Obsidian),
            Some(Material::Obsidian),
            None,
            Some(Material::Obsidian),
            Some(Material::Obsidian),
            Some(Material::Geode),
            Some(Material::Obsidian),
            Some(Material::Geode),
            None,
            Some(Material::Geode),
            None,
            None,
        ],
        // Blueprint 3
        [
            None,
            None,
            None,
            None,
            Some(Material::Clay),
            None,
            None,
            None,
            Some(Material::Clay),
            None,
            None,
            None,
            None,
            Some(Material::Obsidian),
            None,
            None,
            None,
            None,
            Some(Material::Obsidian),
            None,
            None,
            Some(Material::Geode),
            None,
            None,
        ],
        // Blueprint 4
        [
            None,
            None,
            None,
            Some(Material::Ore),
            None,
            Some(Material::Ore),
            None,
            Some(Material::Clay),
            Some(Material::Clay),
            None,
            Some(Material::Clay),
            Some(Material::Obsidian),
            Some(Material::Ore),
            Some(Material::Obsidian),
            Some(Material::Clay),
            Some(Material::Obsidian),
            Some(Material::Obsidian),
            Some(Material::Obsidian),
            Some(Material::Geode),
            Some(Material::Obsidian),
            Some(Material::Obsidian),
            Some(Material::Geode),
            Some(Material::Geode),
            None,
        ],
    ];

    struct SearchState {
        time_allowed: usize,
        most_geodes_cracked: GeodesCracked,
        build_recipe: Vec<Option<Material>>,
        max_robots_needed: RobotInventory,
        max_build_time: HashMap<Material, usize>,
    }
    impl SearchState {
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
                build_recipe: Vec::new(),
                max_robots_needed,
                max_build_time,
            }
        }
    }

    #[derive(Clone)]
    struct ToBuildNext {
        time_to_build: usize,
        to_build: Material,
    }

    #[derive(Clone)]
    struct SearchNode<'a> {
        blueprint: &'a Blueprint,
        time_tracker: TimeTracker<'a>,
        to_build_next: Option<ToBuildNext>,
        build_recipe: Vec<Option<Material>>,
    }
    impl<'a> SearchNode<'a> {
        fn new(blueprint: &'a Blueprint, time_allowed: usize) -> Self {
            Self {
                blueprint,
                time_tracker: TimeTracker::new(&blueprint.robot_costs, time_allowed),
                build_recipe: Vec::with_capacity(time_allowed),
                to_build_next: None,
            }
        }

        fn duplicate(&self, to_build_next: ToBuildNext) -> Self {
            Self {
                blueprint: self.blueprint,
                time_tracker: self.time_tracker.clone(),
                build_recipe: self.build_recipe.clone(),
                to_build_next: Some(to_build_next),
            }
        }
    }
    impl GlobalStateTreeNode for SearchNode<'_> {
        type GlobalState = SearchState;

        fn recurse_action(mut self, global_state: &mut Self::GlobalState) -> NodeAction<Self> {
            // Wait to build our current robot
            if let Some(ref ttb) = self.to_build_next {
                for _ in 0..ttb.time_to_build {
                    self.build_recipe.push(None);
                }
                self.build_recipe.push(Some(ttb.to_build));

                if let Err(gc) = self
                    .time_tracker
                    .wait_to_build_robot(ttb.time_to_build, ttb.to_build)
                    .and_then(|_| self.time_tracker.time_up())
                {
                    // TODO: switch back to best cost search?
                    if GeodesCracked(gc).is_better(&global_state.most_geodes_cracked) {
                        global_state.build_recipe = self.build_recipe.clone();
                    }
                    global_state
                        .most_geodes_cracked
                        .update_if_better(GeodesCracked(gc));
                    //println!("Done with {gc} geodes cracked!");
                    return NodeAction::Stop;
                }
            }

            // See which robots we can actually build right now
            let mut children = Vec::new();

            // TODO: Could do this with iterator filter map
            for to_build in Material::iter().rev() {
                if let Infinitable::Finite(t) = self.time_tracker.time_to_build_robot(&to_build)
                    && self.time_tracker.elapsed_time + t < global_state.time_allowed
                    && self.time_tracker.robots.count_of(&to_build)
                        < global_state.max_robots_needed.count_of(&to_build)
                    && self.time_tracker.elapsed_time + t <= global_state.max_build_time[&to_build]
                    && self.time_tracker.should_build_robot(t, to_build)
                {
                    // TODO
                    /* if !self.time_tracker.should_build_robot(t, to_build) {
                        println!(
                            "Can avoid build of {to_build:?} robot at minute {}",
                            self.time_tracker.elapsed_time + 1
                        );
                    } */

                    // TODO: Give ToBuildNext a constructor
                    children.push(self.duplicate(ToBuildNext {
                        time_to_build: t,
                        to_build,
                    }));
                }
            }

            // If we have nothing to build, then we can just run out the clock
            if children.is_empty() {
                global_state
                    .most_geodes_cracked
                    .update_if_better(GeodesCracked(self.time_tracker.run_out_clock()));
                NodeAction::Stop
            } else {
                NodeAction::Continue(children)
            }
        }
    }

    #[derive(Default, Clone, Copy, Add)]
    struct GeodesCracked(usize);
    impl Metric for GeodesCracked {
        fn is_better(&self, other: &Self) -> bool {
            self.0 > other.0
        }
    }

    // Ugh, need this because [`HashMultiSet`] does not implement [`Hash`].
    #[derive(Deref, DerefMut, Clone, PartialEq, Eq)]
    struct MultiSet<K: Eq + Hash>(HashMultiSet<K>);
    impl<K: Eq + Hash> Hash for MultiSet<K> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            for key in self.0.iter() {
                key.hash(state)
            }
        }
    }
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

    type TimeToBuild = Infinitable<MultiSet<Material>>;
    type RobotCost = MultiSet<Material>;
    type MaterialInventory = MultiSet<Material>;
    type RobotInventory = MultiSet<Material>;

    #[derive(Debug, Clone)]
    struct TimeTracker<'a> {
        robot_costs: &'a RobotCosts,
        time_allowed: usize,
        materials: MaterialInventory,
        robots: RobotInventory,
        elapsed_time: usize,
    }
    impl<'a> TimeTracker<'a> {
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

        pub fn geodes_cracked(&self) -> usize {
            self.materials.count_of(&Material::Geode)
        }

        // Passes one minute of time
        // Does nothing if time is up
        // Returns geodes cracked in during the minute
        // TODO: Return GeodesCracked everywhere maybe, if nothing else makes the intention clearer.
        fn tick(&mut self) -> Result<(), usize> {
            // TODO use self.time_up here if it survives
            if self.elapsed_time < self.time_allowed {
                // Have the robots collect materials
                for robot in self.robots.iter() {
                    self.materials.insert(*robot)
                }

                // Increment time
                self.elapsed_time += 1;

                Ok(())
            } else {
                Err(self.geodes_cracked())
            }
        }

        pub fn time_up(&self) -> Result<(), usize> {
            if self.elapsed_time >= self.time_allowed {
                Err(self.geodes_cracked())
            } else {
                Ok(())
            }
        }

        pub fn can_build_robot(&self, robot_type: &Material) -> bool {
            let robot_cost = &self.robot_costs[robot_type];
            robot_cost
                .distinct_elements()
                .all(|m| self.materials.count_of(m) >= robot_cost.count_of(m))
        }

        // Raw dog, no check whether we can build.
        fn build_robot(&mut self, robot_type: Material) {
            // Spend the materials
            self.materials -= &self.robot_costs[&robot_type];

            // Add the new robot
            self.robots.insert(robot_type);
        }

        pub fn advance_time(&mut self, to_build: Option<Material>) -> Result<(), usize> {
            if DEBUG_PRINT {
                self.time_up()?;
                println!(
                    "============== Minute {} ===============",
                    self.elapsed_time + 1
                );
            }

            if let Some(rt) = to_build
                && !self.can_build_robot(&rt)
            {
                panic!("cannot afford to build a {rt:?} robot!");
            }

            if DEBUG_PRINT {
                if let Some(rt) = to_build {
                    println!("Building a {rt:?} robot");
                    println!();
                }

                println!("Robots: {:?}", self.robots);
            }
            self.tick()?;

            if let Some(rt) = to_build {
                self.build_robot(rt)
            }

            if DEBUG_PRINT {
                println!("Materials: {:?}\n", self.materials);
            }
            Ok(())
        }

        // Returns total geodes cracked if time ran out
        // TODO: Consider taking ToBuildNext if that struct sticks around
        pub fn wait_to_build_robot(
            &mut self,
            time_to_build: usize,
            to_build: Material,
        ) -> Result<(), usize> {
            for _ in 0..time_to_build {
                self.advance_time(None)?;
            }
            self.advance_time(Some(to_build))
        }

        pub fn run_out_clock(&mut self) -> usize {
            loop {
                if let Err(gc) = self.tick() {
                    break gc;
                }
            }
        }

        // TODO: We may never actually care about individual material costs just the max
        pub fn time_to_build_robot(&self, robot_type: &Material) -> Infinitable<usize> {
            let mut times = MultiSet::default();

            for material in Material::iter() {
                let cost = self.robot_costs[robot_type].count_of(&material);
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

        fn time_to_build_important(&self, to_build: &Material) -> Vec<Infinitable<usize>> {
            [Material::Obsidian, Material::Geode]
                .into_iter()
                .filter_map(|rt| (rt != *to_build).then(|| self.time_to_build_robot(&rt)))
                .collect()
        }

        // TODO do we need this?
        // TODO: replace with TimeToBuild
        pub fn should_build_robot(&self, time_to_build: usize, to_build: Material) -> bool {
            let current_ttb = self.time_to_build_important(&to_build);

            let mut future = self.clone();
            let _ = future.wait_to_build_robot(time_to_build, to_build);

            current_ttb
                .into_iter()
                .zip(future.time_to_build_important(&to_build))
                .all(|(c, f)| f <= c)
        }
    }

    #[derive(Debug)]
    pub struct RobotFactory {
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
        pub fn sum_of_quality_levels(&self) -> u64 {
            self.blueprints
                .iter()
                .map(|b| {
                    let mgc = b.largest_geodes_cracked(24);
                    println!("===================== MGC for blueprint {}: {mgc}\n", b.num);
                    u64::from(b.num) * mgc
                })
                .sum()

            // TODO
            //self.blueprints[0].largest_geodes_cracked(24)
        }

        pub fn product_of_most_geodes(&self) -> u64 {
            self.blueprints
                .iter()
                .take(3)
                .map(|b| {
                    let mgc = b.largest_geodes_cracked(32);
                    println!("===================== MGC for blueprint {}: {mgc}\n", b.num);
                    mgc
                })
                .product()
        }
    }

    const DEBUG_PRINT: bool = false;
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
