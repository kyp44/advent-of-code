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
        tree_search::{GlobalStateTreeNode, Metric, NodeAction},
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
            /* let search_state = SearchNode::new(self).traverse_tree(SearchState::new(self));

            // TODO: print recipe
            println!(
                "Recipe: {:?}",
                search_state
                    .build_recipe
                    .into_iter()
                    .map(|rs| rs.built)
                    .collect_vec()
            );

            return Ok(search_state.most_geodes_cracked.0.try_into().unwrap()); */

            // TODO test code
            let mut time_tracker = TimeTracker::new(&self.robot_costs);

            for build in BUILD_RECIPES[usize::from(self.num) - 1] {
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

                //let should_build = time_tracker.should_build_robot();
                let should_build = build;

                /* if let Some(rt) = time_tracker.should_build_robot() {
                    println!("We should build a {rt:?} robot!");
                    println!();
                } */

                if let Some(rt) = should_build {
                    println!("Building a {rt:?} robot");
                    println!();
                }
                println!("Robots: {:?}", time_tracker.robots);
                time_tracker.advance_time(should_build);
                println!("Materials: {:?}", time_tracker.materials);
                println!();
            }

            Ok(time_tracker
                .materials
                .count_of(&Material::Geode)
                .try_into()
                .unwrap())
        }
    }

    const BUILD_RECIPES: &[[Option<Material>; 24]] = &[
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
            Some(Material::Clay),
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
            None,
        ],
    ];

    struct SearchState {
        build_recipe: Vec<RecipeStep>,
        max_robots_needed: RobotInventory,
        most_geodes_cracked: GeodesCracked,
    }
    impl SearchState {
        pub fn new(blueprint: &Blueprint) -> Self {
            let mut max_robots_needed = RobotInventory::default();
            for rt in Material::iter() {
                let max_cost = blueprint
                    .robot_costs
                    .values()
                    .map(|c| c.count_of(&rt))
                    .max()
                    .unwrap();

                max_robots_needed.insert_times(
                    rt,
                    if max_cost > 0 {
                        max_cost
                    } else {
                        TIME_ALLOWED.into()
                    },
                )
            }

            Self {
                build_recipe: Vec::new(),
                max_robots_needed,
                most_geodes_cracked: GeodesCracked(0),
            }
        }
    }

    #[derive(Clone)]
    struct RecipeStep {
        built: Option<Material>,
        materials: MaterialInventory,
    }

    #[derive(Clone)]
    struct SearchNode<'a> {
        blueprint: &'a Blueprint,
        time_tracker: TimeTracker<'a>,
        to_build: Option<Material>,
        build_recipe: Vec<RecipeStep>,
    }
    impl<'a> SearchNode<'a> {
        fn new(blueprint: &'a Blueprint) -> Self {
            Self {
                blueprint,
                time_tracker: TimeTracker::new(&blueprint.robot_costs),
                to_build: None,
                build_recipe: Vec::with_capacity(TIME_ALLOWED.into()),
            }
        }

        fn duplicate(&self, to_build: Option<Material>) -> Self {
            Self {
                blueprint: self.blueprint,
                time_tracker: self.time_tracker.clone(),
                build_recipe: self.build_recipe.clone(),
                to_build,
            }
        }
    }
    impl GlobalStateTreeNode for SearchNode<'_> {
        type GlobalState = SearchState;

        fn recurse_action(mut self, global_state: &mut Self::GlobalState) -> NodeAction<Self> {
            // If time is up, we are done
            if let Some(gc) = self.time_tracker.time_up() {
                println!(
                    "TODO time is up: {}",
                    self.time_tracker.materials.count_of(&Material::Geode)
                );
                let geodes_cracked = GeodesCracked(gc);
                if geodes_cracked.is_better(&global_state.most_geodes_cracked) {
                    global_state
                        .most_geodes_cracked
                        .update_if_better(geodes_cracked);
                    global_state.build_recipe = self.build_recipe;
                }

                return NodeAction::Stop;
            }

            println!(
                "TODO ============== Minute {} ===============",
                self.time_tracker.elapsed_time + 1
            );
            println!("TODO Building a {:?} robot", self.to_build);

            // Optimization to shorten the search time
            if !global_state.build_recipe.is_empty() {
                let best_materials = &global_state.build_recipe[self.build_recipe.len()].materials;
                if self.time_tracker.materials.count_of(&Material::Geode)
                    < best_materials.count_of(&Material::Geode)
                {
                    println!(
                        "TODO no good! us: {:?} global: {:?}",
                        self.time_tracker.materials,
                        global_state.build_recipe[self.build_recipe.len()].materials
                    );
                    return NodeAction::Stop;
                }
            }

            println!("TODO robots: {:?}", self.time_tracker.robots);
            self.build_recipe.push(RecipeStep {
                built: self.to_build,
                materials: self.time_tracker.materials.clone(),
            });
            self.time_tracker.advance_time(self.to_build);
            println!("TODO materials: {:?}", self.time_tracker.materials);

            let mut children = Vec::new();

            for build_rt in Material::iter().rev() {
                // Can we build this robot?
                if self.time_tracker.can_build_robot(&build_rt)
                    && self.time_tracker.robots.count_of(&build_rt)
                        < global_state.max_robots_needed.count_of(&build_rt)
                {
                    children.push(self.duplicate(Some(build_rt)));
                }
            }

            // We always want to build something if we can
            //if children.is_empty() {
            children.push(self.duplicate(None));
            //}

            NodeAction::Continue(children)
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

        pub fn geodes_cracked(&self) -> usize {
            self.materials.count_of(&Material::Geode)
        }

        // Passes one minute of time
        // Does nothing if time is up
        // Returns geodes cracked in during the minute
        fn tick(&mut self) -> bool {
            if self.elapsed_time < TIME_ALLOWED {
                // Have the robots collect materials
                for robot in self.robots.iter() {
                    self.materials.insert(*robot)
                }

                // Increment time
                self.elapsed_time += 1;

                true
            } else {
                false
            }
        }

        pub fn time_up(&self) -> Option<usize> {
            (self.elapsed_time >= TIME_ALLOWED).then_some(self.geodes_cracked())
        }

        pub fn can_build_robot(&self, robot_type: &Material) -> bool {
            let robot_cost = &self.robot_costs[robot_type];
            robot_cost
                .distinct_elements()
                .all(|m| self.materials.count_of(m) >= robot_cost.count_of(m))
        }

        // Will panic if cannot afford to build and also advances time
        fn build_robot(&mut self, robot_type: Material) {
            // Spend the materials
            self.materials -= &self.robot_costs[&robot_type];

            // Add the new robot
            self.robots.insert(robot_type);
        }

        pub fn advance_time(&mut self, to_build: Option<Material>) {
            if let Some(rt) = to_build
                && !self.can_build_robot(&rt)
            {
                panic!("Cannot afford to build a {rt:?} robot!");
            }

            self.tick();

            if let Some(rt) = to_build {
                self.build_robot(rt)
            }
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
                        Ordering::Equal => {
                            if hasten_rt == Material::Clay && new_time.is_finite() {
                                println!("Dare we build a {robot_type:?} robot?!");
                            }
                        }
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
            self.blueprints[1].largest_geodes_cracked()
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
