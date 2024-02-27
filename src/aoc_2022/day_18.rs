use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "2,2,2
1,2,2
3,2,2
2,1,2
2,3,2
2,2,1
2,2,3
2,2,4
2,2,6
1,2,5
3,2,5
2,1,5
2,3,5";
            answers = unsigned![64, 58];
        }
        actual_answers = unsigned![3470, 1986];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{
        parse::trim,
        tree_search::{GlobalStateTreeNode, NodeAction},
    };
    use cgmath::{Point3, Vector3};
    use derive_new::new;
    use itertools::{iproduct, Itertools};
    use nom::{bytes::complete::tag, combinator::map, multi::separated_list1};
    use std::{collections::HashSet, marker::PhantomData, ops::RangeInclusive};

    /// The 3D points for the cube locations.
    type Point = Point3<i16>;
    /// The 3D vectors used to translate cube locations.
    type Vector = Vector3<i16>;

    /// A single cube, which can be parsed from text input.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct Cube {
        /// The location of the cube.
        location: Point,
    }
    impl Parsable<'_> for Cube {
        fn parser(input: &'_ str) -> NomParseResult<&str, Self> {
            map(
                trim(
                    false,
                    separated_list1(tag(","), nom::character::complete::i16),
                ),
                |v| Cube {
                    location: Point::new(v[0], v[1], v[2]),
                },
            )(input)
        }
    }
    impl From<Point> for Cube {
        fn from(value: Point) -> Self {
            Self { location: value }
        }
    }
    impl Cube {
        /// Returns an iterator over the six adjacent cubes to this cube,
        /// noting that this does not include diagonal cubes in any dimension.
        pub fn neighbors(&self) -> impl Iterator<Item = Cube> + '_ {
            let ds = -1..=1;
            iproduct!(ds.clone(), ds.clone(), ds).filter_map(|tup| {
                let d = Vector::from(tup);

                (d.manhattan_len() == 1).then(|| Cube {
                    location: self.location + d,
                })
            })
        }
    }

    /// The global state when recursively determining 3D regions.
    ///
    /// This also keeps track of the cubes in each region being determined so far.
    struct RegionSearchState<'a> {
        /// The set of cubes comprising the droplet itself.
        droplet_cubes: &'a HashSet<Cube>,
        /// The inclusive bounds of the droplet for each dimension.
        bounds: Point3<RangeInclusive<i16>>,
        /// The set of cubes comprising the regions outside the droplet.
        outside_cubes: HashSet<Cube>,
        /// The set of cubes comprising air pockets within the droplet.
        pocket_cubes: HashSet<Cube>,
        /// The set of cubes comprising the current region thus far.
        current_region: HashSet<Cube>,
        /// Whether the current region has been identified as being outside the droplet.
        outside_region: bool,
    }
    impl<'a> RegionSearchState<'a> {
        /// Creates a new search state given the set of droplet cubes.
        ///
        /// The new search state begins with no regions identified except the droplet.
        pub fn new(droplet: &'a HashSet<Cube>) -> AocResult<Self> {
            let cr = |mapper: fn(&Point) -> i16| match droplet
                .iter()
                .map(|c| mapper(&c.location))
                .minmax()
            {
                itertools::MinMaxResult::MinMax(a, b) => Ok(a..=b),
                _ => Err(AocError::Process(
                    "The droplet must comprise more than one cube!".into(),
                )),
            };

            Ok(Self {
                droplet_cubes: droplet,
                bounds: Point3::new(cr(|p| p.x)?, cr(|p| p.y)?, cr(|p| p.z)?),
                outside_cubes: Default::default(),
                pocket_cubes: Default::default(),
                current_region: Default::default(),
                outside_region: false,
            })
        }

        /// Resets the current region.
        fn reset(&mut self) {
            self.current_region.clear();
            self.outside_region = false;
        }

        /// Returns whether a given `cube` is within the bounds of the droplet/problem.
        fn in_bounds(&self, cube: &Cube) -> bool {
            let p = &cube.location;
            self.bounds.x.contains(&p.x)
                && self.bounds.y.contains(&p.y)
                && self.bounds.z.contains(&p.z)
        }

        /// Returns whether a `cube` is in one of the currently known regions, including
        /// the current region.
        fn known_cube(&self, cube: &Cube) -> bool {
            self.current_region.contains(cube)
                || self.droplet_cubes.contains(cube)
                || self.outside_cubes.contains(cube)
                || self.pocket_cubes.contains(cube)
        }

        /// Creates a new search node for the `cube` only if the cube is not currently
        /// in a known region or out of bounds.
        ///
        /// If the cube is out of bounds, the current region is flagged as being outside
        /// the droplet.
        fn new_node(&mut self, cube: Cube) -> Option<RegionSearchNode<'a>> {
            let out_of_bounds = !self.in_bounds(&cube);

            if self.known_cube(&cube) || out_of_bounds {
                if out_of_bounds {
                    self.outside_region = true;
                }
                None
            } else {
                Some(RegionSearchNode::new(cube))
            }
        }

        /// Returns an iterator over every cube within the droplet bounding box.
        fn all_cubes(&self) -> impl Iterator<Item = Cube> {
            iproduct!(
                self.bounds.z.clone(),
                self.bounds.y.clone(),
                self.bounds.x.clone()
            )
            .map(|(z, y, x)| Cube {
                location: Point::new(x, y, z),
            })
        }

        /// Iterates over every cube in the bounding box and classifies them as being
        /// within an outside region, the droplet itself, or an internal air
        /// pocket region.
        ///
        /// This is done by recursively building out a region for any yet unclassified
        /// starting cubes.
        /// Returns the set of cubes that are in any pocket region.
        pub fn pocket_cubes(mut self) -> HashSet<Cube> {
            for start_cube in self.all_cubes() {
                if let Some(node) = self.new_node(start_cube) {
                    self = node.traverse_tree(self);

                    // The current region is guaranteed to at least contain the starting cube
                    if self.outside_region {
                        self.outside_cubes.extend(self.current_region.drain());
                    } else {
                        self.pocket_cubes.extend(self.current_region.drain());
                    }
                }

                self.reset();
            }

            self.pocket_cubes
        }
    }

    /// A search node when determining all the cubes within a region.
    #[derive(new)]
    struct RegionSearchNode<'a> {
        /// The current cube being processed.
        cube: Cube,
        /// Phantom data that allows this node to have a lifetime, which is needed because
        /// the [`RegionSearchState`] requires a lifetime.
        #[new(default)]
        _phantom: PhantomData<&'a str>,
    }
    impl<'a> GlobalStateTreeNode for RegionSearchNode<'a> {
        type GlobalState = RegionSearchState<'a>;

        fn recurse_action(self, global_state: &mut Self::GlobalState) -> NodeAction<Self> {
            // Add to the current region
            global_state.current_region.insert(self.cube);

            // Build child nodes
            let children = self
                .cube
                .neighbors()
                .filter_map(|c| global_state.new_node(c))
                .collect_vec();

            if children.is_empty() {
                NodeAction::Stop
            } else {
                NodeAction::Continue(children)
            }
        }
    }

    /// The droplet, which can be parsed from text input.
    pub struct Droplet {
        /// The set of cubes that comprise the droplet.
        cubes: HashSet<Cube>,
    }
    impl FromStr for Droplet {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                cubes: Cube::gather(s.trim().lines())?.into_iter().collect(),
            })
        }
    }
    impl Droplet {
        /// Determines the surface area of the droplet, optionally excluding the
        /// additional surface area due to internal air pockets (part two).
        pub fn surface_area(&self, exclude_air_pockets: bool) -> AocResult<u64> {
            // We only need to determine regions if excluding air pocket surface area
            let pocket_cubes = if exclude_air_pockets {
                RegionSearchState::new(&self.cubes)?.pocket_cubes()
            } else {
                HashSet::new()
            };

            Ok(self
                .cubes
                .iter()
                .map(|cube| {
                    6 - cube.neighbors().filter_count::<u64>(|nc| {
                        self.cubes.contains(nc) || pocket_cubes.contains(nc)
                    })
                })
                .sum())
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 18,
    name: "Boiling Boulders",
    preprocessor: Some(|input| Ok(Box::new(Droplet::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<Droplet>()?.surface_area(false)?.into())
        },
        // Part two
        |input| {
            // Process
            Ok(input.expect_data::<Droplet>()?.surface_area(true)?.into())
        },
    ],
};
