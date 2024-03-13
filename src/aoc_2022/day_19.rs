use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.
    Blueprint 2: Each ore robot costs 2 ore. Each clay robot costs 3 ore. Each obsidian robot costs 3 ore and 8 clay. Each geode robot costs 3 ore and 12 obsidian.";
            answers = unsigned![33];
        }
        actual_answers = unsigned![123];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{
        parse::trim,
        tree_search::{ApplyNodeAction, BestCostChild, BestCostTreeNode, Metric},
    };
    use derive_more::{Add, Deref, DerefMut};
    use infinitable::Infinitable;
    use itertools::Itertools;
    use multiset::HashMultiSet;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::space1,
        combinator::map,
        multi::{many_m_n, separated_list1},
        sequence::{delimited, pair, separated_pair, terminated},
    };
    use std::{cmp::Ordering, collections::HashMap, hash::Hash};
    use strum::IntoEnumIterator;
    use strum_macros::EnumIter;

    const NUM_ROBOTS: usize = 4;
    // In minutes
    const TIME_ALLOWED: u8 = 24;

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
        pub fn largest_geodes_cracked(&self) -> AocResult<u64> {
            //SearchNode::new(self).traverse_tree().map(|g| g.0.into())

            // TODO test code
            let mut time_tracker = TimeTracker::new(&self.robot_costs);

            for build in BUILD_RECIPE {
                println!("== Minute {} ==", time_tracker.elapsed_time + 1);

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

                let should_build = time_tracker.should_build_robot();

                if let Some(rt) = time_tracker.should_build_robot() {
                    println!("We should build a {rt:?} robot!");
                }
                println!();

                fn print_robots(time_tracker: &TimeTracker) {
                    println!("Robots: {:?}", time_tracker.robots);
                }

                fn print_materials(time_tracker: &TimeTracker) {
                    println!("Materials: {:?}", time_tracker.materials);
                }

                match should_build {
                    Some(rt) => {
                        let robot_cost = &self.robot_costs[&rt];

                        println!("Spending {robot_cost:?} to build a {rt:?} robot",);
                        print_robots(&time_tracker);
                        time_tracker.tick();
                        time_tracker.build_robot(rt);
                        print_materials(&time_tracker);
                    }
                    None => {
                        print_robots(&time_tracker);
                        time_tracker.tick();
                        print_materials(&time_tracker);
                    }
                }

                println!();
            }

            Ok(time_tracker
                .materials
                .count_of(&Material::Geode)
                .try_into()
                .unwrap())
        }
    }

    const BUILD_RECIPE: [Option<Material>; 24] = [
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
    ];

    /* #[derive(Clone)]
    struct SearchNode<'a> {
        blueprint: &'a Blueprint,
        time_tracker: TimeTracker,
        to_build: Option<&'a RobotCost>,
    }
    impl PartialEq for SearchNode<'_> {
        fn eq(&self, other: &Self) -> bool {
            self.time_tracker == other.time_tracker
        }
    }
    impl Eq for SearchNode<'_> {}
    impl Hash for SearchNode<'_> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.time_tracker.hash(state);
        }
    }
    impl<'a> SearchNode<'a> {
        fn new(blueprint: &'a Blueprint) -> Self {
            Self {
                blueprint,
                time_tracker: TimeTracker::default(),
                to_build: None,
            }
        }

        fn duplicate(&self, to_build: Option<&'a RobotCost>) -> Self {
            Self {
                blueprint: self.blueprint,
                time_tracker: self.time_tracker.clone(),
                to_build,
            }
        }
    }
    impl BestCostTreeNode for SearchNode<'_> {
        type Metric = GeodesCracked;

        fn recurse_action(&mut self) -> ApplyNodeAction<BestCostChild<Self>> {
            // If time is up, we are done
            if self.time_tracker.time_up() {
                println!(
                    "TODO time is up: {}",
                    self.time_tracker.materials.count_of(&Material::Geode)
                );
                return ApplyNodeAction::Stop(true);
            }

            println!(
                "TODO ============== Minute {} ===============",
                self.time_tracker.elapsed_time + 1
            );

            // Tick time and see how many geodes were opened.
            let geodes = GeodesCracked(self.time_tracker.tick());

            println!("TODO robots: {:?}", self.time_tracker.robots);

            if let Some(robot_cost) = self.to_build {
                self.time_tracker.build_robot(robot_cost);
                println!("TODO building a robot: {:?}", robot_cost.robot_type);
            }

            println!("TODO materials: {:?}", self.time_tracker.materials);

            let mut children = Vec::new();
            let mut must_build = false;

            for robot_cost in self.blueprint.robot_costs.iter() {
                // Can we build this robot?
                if self.time_tracker.can_build_robot(robot_cost) {
                    children.push(BestCostChild::new(self.duplicate(Some(robot_cost)), geodes));

                    // We must build only this robot if one has never been built
                    // because this just makes good sense.
                    if !self.time_tracker.robots.contains(&robot_cost.robot_type) {
                        must_build = true;
                        break;
                    }
                }
            }

            // Unless we must build, run a scenario where we do not build any
            if !must_build {
                children.push(BestCostChild::new(self.duplicate(None), geodes));
            }

            ApplyNodeAction::Continue(children)
        }
    }

    #[derive(Default, Clone, Copy, Add)]
    struct GeodesCracked(u8);
    impl Metric for GeodesCracked {
        fn is_better(&self, other: &Self) -> bool {
            self.0 > other.0
        }
    } */

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
        materials: MaterialInventory,
        robots: RobotInventory,
        elapsed_time: u8,
    }
    impl<'a> TimeTracker<'a> {
        pub fn new(robot_costs: &'a RobotCosts) -> Self {
            // We start with one ore-collecting robot
            let mut robots = RobotInventory::default();
            robots.insert(Material::Ore);

            Self {
                robot_costs,
                materials: MaterialInventory::default(),
                robots,
                elapsed_time: 0,
            }
        }

        // Passes one minute of time
        // Does nothing if time is up
        // Returns geodes cracked in during the minute
        pub fn tick(&mut self) -> u8 {
            if self.elapsed_time < TIME_ALLOWED {
                // Have the robots collect materials
                for robot in self.robots.iter() {
                    self.materials.insert(*robot)
                }

                // Increment time
                self.elapsed_time += 1;

                self.robots.count_of(&Material::Geode).try_into().unwrap()
            } else {
                0
            }
        }

        pub fn time_up(&self) -> bool {
            self.elapsed_time >= TIME_ALLOWED
        }

        pub fn can_build_robot(&self, robot_type: &Material) -> bool {
            let robot_cost = &self.robot_costs[robot_type];
            robot_cost
                .distinct_elements()
                .all(|m| self.materials.count_of(m) >= robot_cost.count_of(m))
        }

        // Will panic if cannot afford to build
        pub fn build_robot(&mut self, robot_type: Material) {
            // Spend the materials
            self.materials -= &self.robot_costs[&robot_type];

            // Add the new robot
            self.robots.insert(robot_type);
        }

        // TODO: We may never actually care about individual material costs just the max
        pub fn time_to_build_robot(&self, robot_type: &Material) -> TimeToBuild {
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

            Infinitable::Finite(times)
        }

        pub fn should_build_robot(&self) -> Option<Material> {
            // TODO: Maybe make an extension trait for Infinitable?
            fn annoying(ttb: &TimeToBuild) -> Infinitable<usize> {
                match ttb {
                    Infinitable::Finite(ref t) => Infinitable::Finite(t.max_count()),
                    _ => Infinitable::Infinity,
                }
            }

            for robot_type in Material::iter().rev().filter(|rt| self.can_build_robot(rt)) {
                // We always want to build a geode robot if possible
                if robot_type == Material::Geode {
                    return Some(robot_type);
                }

                // Otherwise, does it hasten the building of another robot?
                for hasten_rt in Material::iter().rev() {
                    let current_time = annoying(&self.time_to_build_robot(&hasten_rt));

                    // If we were to build it, does it lessen the time?
                    let mut future = self.clone();
                    future.tick();
                    future.build_robot(robot_type);

                    let new_time =
                        annoying(&future.time_to_build_robot(&hasten_rt)) + Infinitable::Finite(1);

                    match new_time.cmp(&current_time) {
                        Ordering::Less => return Some(robot_type),
                        Ordering::Equal => {}
                        Ordering::Greater => break,
                    }
                }
            }

            None
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
        pub fn sum_of_quality_levels(&self) -> AocResult<u64> {
            println!(
                "TODO answer: {}",
                self.blueprints[1].largest_geodes_cracked()?
            );

            Ok(0)
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
                .sum_of_quality_levels()?
                .into())
        },
    ],
};
