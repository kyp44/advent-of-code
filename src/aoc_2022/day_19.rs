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
    use std::collections::HashMap;

    use super::*;
    use aoc::parse::trim;
    use multiset::HashMultiSet;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::space1,
        combinator::map,
        multi::{many_m_n, separated_list1},
        sequence::{delimited, pair, separated_pair, terminated},
    };

    const NUM_ROBOTS: usize = 4;
    // In minutes
    const TIME_ALLOWED: u8 = 24;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    struct Cost {
        material: Material,
        cost: u8,
    }
    impl Parsable<'_> for Cost {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(nom::character::complete::u8, space1, Material::parser),
                |(cost, material)| Self { material, cost },
            )(input)
        }
    }

    #[derive(Debug)]
    struct RobotCost {
        robot: Material,
        cost: Vec<Cost>,
    }
    impl Parsable<'_> for RobotCost {
        fn parser(input: &'_ str) -> NomParseResult<&str, Self> {
            map(
                pair(
                    delimited(
                        trim(false, tag("Each")),
                        Material::parser,
                        trim(false, tag("robot costs")),
                    ),
                    terminated(
                        separated_list1(trim(false, tag("and")), Cost::parser),
                        trim(false, tag(".")),
                    ),
                ),
                |(robot, cost)| Self { robot, cost },
            )(input)
        }
    }
    impl RobotCost {
        pub fn can_build(&self, materials: &MaterialInventory) -> bool {
            self.cost
                .iter()
                .all(|cost| materials.count_of(&cost.material) >= cost.cost.into())
        }

        // Will panic if cannot afford to build
        pub fn build(&self, materials: &mut MaterialInventory, robots: &mut RobotInventory) {
            // Spend the materials
            for cost in self.cost.iter() {
                materials.remove_times(&cost.material, cost.cost.into());
            }

            // Add the new robot
            robots.insert(self.robot);
        }
    }

    #[derive(Debug)]
    struct Blueprint {
        num: u8,
        robot_costs: [RobotCost; NUM_ROBOTS],
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
                    many_m_n(NUM_ROBOTS, NUM_ROBOTS, RobotCost::parser),
                ),
                |(num, robot_costs)| Self {
                    num,
                    robot_costs: robot_costs.try_into().unwrap(),
                },
            )(input)
        }
    }
    impl Blueprint {
        pub fn largest_geodes_cracked(&self) -> u64 {
            69
        }
    }

    type MaterialInventory = HashMultiSet<Material>;
    type RobotInventory = HashMultiSet<Material>;

    #[derive(Debug)]
    struct TimeTracker {
        pub materials: MaterialInventory,
        pub robots: RobotInventory,
        elapsed_time: u8,
    }
    impl TimeTracker {
        // Passes one minute of time
        // Does nothing if time is up
        pub fn tick(&mut self) {
            if self.elapsed_time < TIME_ALLOWED {
                // Have the robots collect materials
                for robot in self.robots.iter() {
                    self.materials.insert(*robot)
                }

                // Increment time
                self.elapsed_time += 1;
            }
        }

        pub fn time_up(&self) -> bool {
            self.elapsed_time >= TIME_ALLOWED
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
            println!(
                "TODO answer: {}",
                self.blueprints[0].largest_geodes_cracked()
            );

            0
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
    ],
};
