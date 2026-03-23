use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "root@ebhq-gridcenter# df -h
Filesystem            Size  Used  Avail  Use%
/dev/grid/node-x0-y0   10T    8T     2T   80%
/dev/grid/node-x0-y1   11T    6T     5T   54%
/dev/grid/node-x0-y2   32T   28T     4T   87%
/dev/grid/node-x1-y0    9T    7T     2T   77%
/dev/grid/node-x1-y1    8T    0T     8T    0%
/dev/grid/node-x1-y2   11T    7T     4T   63%
/dev/grid/node-x2-y0   10T    6T     4T   60%
/dev/grid/node-x2-y1    9T    8T     1T   88%
/dev/grid/node-x2-y2    9T    6T     3T   66%";
            answers = unsigned![7, 7];
        }
        actual_answers = unsigned![892, 227];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use derive_more::{Add, AddAssign, From, PartialEq, Sub, SubAssign};
    use euclid::Vector2D;
    use itertools::{Itertools, iproduct, process_results};
    use nom::{bytes::complete::tag, combinator::map, sequence::terminated};
    use std::{
        collections::{HashMap, hash_map::Entry},
        fmt::Write,
    };

    /// An amount of data in terabytes.
    ///
    /// Can be parsed from text input.
    #[derive(
        Clone, Copy, Default, From, Add, AddAssign, Sub, SubAssign, PartialEq, Eq, PartialOrd, Ord,
    )]
    struct TeraBytes(pub u16);
    impl Parsable for TeraBytes {
        type Parsed<'a> = Self;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(
                trim(false, terminated(nom::character::complete::u16, tag("T"))),
                Self::from,
            )
            .parse(input)
        }
    }
    impl std::fmt::Debug for TeraBytes {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:02}T", self.0)
        }
    }

    /// A storage node in the [`Cluster`].
    ///
    /// Can be parsed from text input.
    #[derive(Clone, Default)]
    struct StorageNode {
        /// The point in the cluster grid where this node resides.
        point: GridPoint,
        /// The amount of data used.
        used: TeraBytes,
        /// The amount of free space.
        available: TeraBytes,
        /// Whether this node contains the goal data we want to access.
        goal_data: bool,
    }
    impl Parsable for StorageNode {
        type Parsed<'a> = Self;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            use nom::character::complete::usize as pusize;

            map(
                (
                    map(
                        (tag("/dev/grid/node-x"), pusize, tag("-y"), pusize),
                        |(_, x, _, y)| GridPoint::new(x, y),
                    ),
                    TeraBytes::parser,
                    TeraBytes::parser,
                    TeraBytes::parser,
                ),
                |(point, size, used, available)| {
                    assert_eq!(size, used + available);
                    Self {
                        point,
                        used,
                        available,
                        goal_data: false,
                    }
                },
            )
            .parse(input)
        }
    }
    impl StorageNode {
        /// Returns whether or not this node is completely empty.
        pub fn is_empty(&self) -> bool {
            self.used.0 == 0
        }

        /// Returns the total storage capacity of this node.
        pub fn capacity(&self) -> TeraBytes {
            self.used + self.available
        }

        /// Returns whether or not this data can be moved into another storage
        /// node.
        pub fn can_move_into(&self, other: &Self) -> bool {
            !self.is_empty() && !std::ptr::eq(self, other) && self.used <= other.available
        }

        /// Returns whether or not this data could be moved into another storage
        /// node if the other node were empty.
        pub fn could_move_into(&self, other: &Self) -> bool {
            !std::ptr::eq(self, other) && self.used <= other.capacity()
        }

        /// Copies the data into this node from another node if possible.
        ///
        /// Returns whether or not the data was copied.
        pub fn copy_from(&mut self, other: &Self) -> bool {
            (self.available >= other.used).and_do(|| {
                self.used += other.used;
                self.available -= other.used;
                self.goal_data = other.goal_data;
            })
        }

        /// Deletes all data on this node, freeing up all of its space.
        pub fn delete_data(&mut self) {
            self.available += self.used;
            self.used = 0.into();
            self.goal_data = false;
        }
    }
    impl std::fmt::Debug for StorageNode {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let open_char = if self.goal_data { '[' } else { '-' };
            let close_char = if self.goal_data { ']' } else { '-' };
            write!(
                f,
                "{}{:?}/{:?}{}",
                open_char, self.used, self.available, close_char
            )
        }
    }

    /// The abstract type of a storage node.
    #[derive(Clone, Copy, Default)]
    enum NodeType {
        /// The node is mostly full, meaning it is none of the other types.
        #[default]
        MostlyFull,
        /// The node is mostly empty, meaning that at least one neighboring
        /// node could copy its data here.
        MostlyEmpty,
        /// Contains a very large amount of data, meaning that there is  at
        /// least one neighboring node that is not large enough to
        /// contain the data from this node even if it were empty.
        VeryLarge,
        /// This node contains the goal data we want to access.
        Goal,
    }
    impl NodeType {
        /// Determines the type of a node for a `cluster` at the given `point`.
        pub fn new(cluster: &Cluster, point: &GridPoint) -> Self {
            let storage_node = cluster.grid.get(point);

            if cluster.nodes_can_move_from(point).next().is_some() {
                Self::MostlyEmpty
            } else if storage_node.goal_data {
                Self::Goal
            } else if cluster
                .grid
                .neighbor_points(point, false, false)
                .any(|np| storage_node.used > cluster.grid.get(&np).capacity())
            {
                Self::VeryLarge
            } else {
                Self::MostlyFull
            }
        }
    }
    impl std::fmt::Debug for NodeType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_char(match self {
                Self::MostlyFull => '.',
                Self::MostlyEmpty => '_',
                Self::VeryLarge => '#',
                Self::Goal => 'G',
            })
        }
    }

    /// A cluster of storage nodes.
    ///
    /// Can be parsed from text input.
    #[derive(Clone)]
    pub struct Cluster {
        /// The grid of storage nodes.
        grid: Grid<StorageNode>,
    }
    impl FromStr for Cluster {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let nodes = StorageNode::gather(s.lines().skip(2))?;

            // Build a map of the grid point to the node data
            let mut map = HashMap::new();
            for node in nodes {
                if let Entry::Vacant(vacant) = map.entry(node.point) {
                    vacant.insert(node);
                } else {
                    return Err(AocError::InvalidInput(
                        format!("The node at {:?} appears more than once", node.point).into(),
                    ));
                }
            }

            // Find the grid size
            let size = GridSize::new(
                map.keys().map(|p| p.x).max().unwrap() + 1,
                map.keys().map(|p| p.y).max().unwrap() + 1,
            );

            // Build the grid
            let mut grid = Grid::default(size);
            for point in grid.all_points() {
                grid.set(
                    &point,
                    map.remove(&point).ok_or_else(|| {
                        AocError::InvalidInput(format!("Node {:?} is missing", point).into())
                    })?,
                );
            }
            grid.get_mut(&GridPoint::new(grid.size().width - 1, 0))
                .goal_data = true;

            Ok(Self { grid })
        }
    }
    impl Cluster {
        /// Calculates and returns the number of viable pairs between which data
        /// transfer can occur.
        pub fn num_viable_pairs(&self) -> u64 {
            iproduct!(self.grid.all_values(), self.grid.all_values())
                .filter_count(|(a, b)| a.can_move_into(b))
        }

        /// Returns the grid point of the node that is empty.
        ///
        /// An `Err` is returned if not exactly one node is empty.
        fn empty_point(&self) -> AocResult<GridPoint> {
            let mut iter = self
                .grid
                .all_points()
                .filter(|p| self.nodes_can_move_from(p).next().is_some());

            let empty_point = iter.next().ok_or(AocError::NoSolution)?;
            match iter.next() {
                Some(_) => Err(AocError::Process(
                    "More than one node with free space!".into(),
                )),
                None => Ok(empty_point),
            }
        }

        /// Returns the point of the node containing the goal data.
        fn goal_point(&self) -> GridPoint {
            self.grid
                .all_points()
                .find(|p| self.grid.get(p).goal_data)
                .expect("More than one goal point found!")
        }

        /// Moves the data `from` one node `to` another if possible.
        ///
        /// Returns `Ok` if the data was moved and `Err` if it could not be.
        fn move_data(&mut self, from: &GridPoint, to: &GridPoint) -> AocResult<()> {
            let from_node = self.grid.get(from).clone();
            if self.grid.get_mut(to).copy_from(&from_node) {
                self.grid.get_mut(from).delete_data();
                Ok(())
            } else {
                Err(AocError::Process(
                    format!(
                        "Was unable to move data from node {:?} to node {:?}",
                        from_node.point,
                        self.grid.get(to).point
                    )
                    .into(),
                ))
            }
        }

        /// Returns an [`Iterator`] of points of neighbor nodes whose data can
        /// be moved into this.
        fn nodes_can_move_from(&self, point: &GridPoint) -> impl Iterator<Item = GridPoint> {
            self.grid
                .neighbor_points(point, false, false)
                .filter(|np| self.grid.get(np).can_move_into(self.grid.get(point)))
        }

        /// Returns a corresponding grid of node types representing the cluster.
        fn type_grid(&self) -> Grid<NodeType> {
            let mut grid = Grid::default(self.grid.size());

            for point in grid.all_points() {
                grid.set(&point, NodeType::new(self, &point))
            }

            grid
        }

        /// Runs an algorithm to determine the minimum number of steps to move
        /// the goal data to the node at `(0, 0)`.
        ///
        /// To move the goal data from the right to the left, the approach does
        /// the following in alternating order:
        ///
        /// * Use the `A*` algorithm to determine the shortest path from the
        ///   empty node to the node immediately to the left of the node
        ///   containing the goal data
        /// * Move the goal data into the empty node to its left.
        ///
        /// This terminates once the goal data is all the way to the left at
        /// point `(0, 0)`.
        pub fn min_moves_to_access_data(self) -> AocResult<u64> {
            process_results(self, |iter| iter.map(|n| u64::try_from(n).unwrap()).sum())
        }
    }
    impl std::fmt::Debug for Cluster {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.type_grid().fmt(f)
        }
    }
    impl Iterator for Cluster {
        type Item = AocResult<usize>;

        fn next(&mut self) -> Option<Self::Item> {
            let goal_point = self.goal_point();

            if goal_point == GridPoint::zero() {
                // We are done!
                return None;
            }

            let res: AocResult<usize> = try {
                // The point to the left of the goal
                let target_point = self
                    .grid
                    .bounded_point(&(goal_point.try_cast::<isize>().unwrap() - Vector2D::unit_x()))
                    .ok_or(AocError::Process(
                        "The goal node somehow got to all the way to the left!".into(),
                    ))?;
                let empty_point = self.empty_point()?;

                if empty_point == target_point {
                    // Right next to the goal data so move it over
                    self.move_data(&goal_point, &empty_point)?;
                    1
                } else {
                    // Find the shortest path of the space to the left of the goal data
                    let (graph, grid) = self.grid.as_graph(false, |sn, nsn| {
                        (!nsn.goal_data && sn.could_move_into(nsn)).then_some(())
                    });
                    let target_idx = *grid.get(&target_point);

                    let (distance, path) = petgraph::algo::astar(
                        &graph,
                        *grid.get(&empty_point),
                        |ni| ni == target_idx,
                        |_| 1usize,
                        |ni| {
                            let node_point = graph.node_weight(ni).unwrap().point;

                            (node_point.max(target_point) - node_point.min(target_point))
                                .manhattan_len()
                        },
                    )
                    .ok_or(AocError::NoSolution)?;

                    // Execute path, moving from one node into another
                    for (to_idx, from_idx) in path.into_iter().tuple_windows() {
                        self.move_data(
                            &graph.node_weight(from_idx).unwrap().point,
                            &graph.node_weight(to_idx).unwrap().point,
                        )?;
                    }

                    distance
                }
            };

            Some(res)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 22,
    name: "Grid Computing",
    preprocessor: Some(|input| Ok(Box::new(Cluster::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<Cluster>()?.num_viable_pairs().into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Cluster>()?
                .clone()
                .min_moves_to_access_data()?
                .into())
        },
    ],
};
