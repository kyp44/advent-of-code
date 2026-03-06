use aoc::{
    prelude::*,
    tree_search::{BestCostTreeNode, Steps},
};

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;
    use aoc::solution::Answer::{String, Unsigned};

    solution_tests! {
        example {
            input = "ihgpwlah";
            answers = answers![String("DDRRRD".into()), Unsigned(370)];
        }
        example {
            input = "kglvqrro";
            answers = answers![String("DDUDRLRRUDRD".into()), Unsigned(492)];
        }
        example {
            input = "ulqzkmiv";
            answers = answers![String("DRURDRUDDLLDLUURRDULRLDUUDDDRR".into()), Unsigned(830)];
        }
        actual_answers = answers![String("RDURRDDLRD".into()), Unsigned(526)];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::tree_search::{ApplyNodeAction, BestCostChild, Metric};
    use euclid::{Box2D, Point2D, Size2D, Vector2D};
    use std::{fmt::Write, marker::PhantomData, ops::Add};
    use strum::{EnumIter, IntoEnumIterator};

    /// A set of room coordinates.
    type Point = Point2D<i8, GridSpace>;
    /// The corresponding vector type for adding to [`Point`]s.
    type Vector = Vector2D<i8, GridSpace>;

    /// A direction of a potential door.
    #[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq, Hash)]
    pub enum Direction {
        /// Up (negative `y`), represented by `U`.
        Up,
        /// Down (positive `y`), represented by `D`.
        Down,
        /// Left (negative `x`), represented by `L`.
        Left,
        /// Right (positive `x`), represented by `R`.
        Right,
    }
    impl Direction {
        /// Returns the unit vector corresponding with this direction.
        pub fn as_vector(&self) -> Vector {
            match self {
                Self::Up => -Vector::unit_y(),
                Self::Down => Vector::unit_y(),
                Self::Left => -Vector::unit_x(),
                Self::Right => Vector::unit_x(),
            }
        }
    }
    impl std::fmt::Display for Direction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_char(match self {
                Direction::Up => 'U',
                Direction::Down => 'D',
                Direction::Left => 'L',
                Direction::Right => 'R',
            })
        }
    }

    /// A path to a room starting from the [`Vault::starting_room`].
    #[derive(Clone, Default, PartialEq, Eq, Hash)]
    pub struct Path(Vec<Direction>);
    impl std::fmt::Debug for Path {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_tuple("Path").field(&self.0).finish()
        }
    }
    impl std::fmt::Display for Path {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for dir in self.0.iter() {
                dir.fmt(f)?
            }
            Ok(())
        }
    }
    impl Path {
        /// Appends a direction and returns the resulting path.
        pub fn append(&self, dir: Direction) -> Self {
            let mut new = self.0.clone();
            new.push(dir);
            Self(new)
        }

        /// Returns a string representing the path, as a sequence of directions
        /// characters.
        pub fn as_string(&self) -> String {
            format!("{self}")
        }
    }

    /// Represents the all of the rooms in the vault area.
    #[derive(Debug)]
    pub struct Vault {
        /// The passcode to determine door access for a [`Path`].
        passcode: String,
        /// The bounding box for the room coordinates.
        bounding_box: Box2D<i8, GridSpace>,
    }
    impl Vault {
        /// Creates a new [`Vault`] area with the `passcode` and the standard
        /// number of rooms.
        pub fn new(passcode: &str) -> Self {
            Self {
                passcode: passcode.into(),
                bounding_box: Box2D::from_size(Size2D::new(4, 4)),
            }
        }

        /// Returns the starting room in the upper left.
        ///
        /// The type `M` is the [`Metric`] used to determine the desired
        /// path through the vault rooms.
        pub fn starting_room<M>(&self) -> CurrentRoom<'_, M> {
            CurrentRoom {
                vault: self,
                point: Point::zero(),
                path_here: Path::default(),
                _phant: PhantomData,
            }
        }
    }

    /// A hash used determine door access.
    struct DoorHash([u8; 4]);
    impl From<&str> for DoorHash {
        /// Hashes the `value` string.
        fn from(value: &str) -> Self {
            let bytes = md5::compute(value.as_bytes()).0;
            Self([
                bytes[0] >> 4,
                bytes[0] & 0x0F,
                bytes[1] >> 4,
                bytes[1] & 0x0F,
            ])
        }
    }
    impl DoorHash {
        /// Returns an [`Iterator`] for the [`Direction`]s in which the doors
        /// are open.
        ///
        /// Note that this only uses the hash and does not take into account the
        /// current room and which doors are available in that room.
        pub fn open_doors(&self) -> impl Iterator<Item = Direction> {
            Direction::iter()
                .zip(self.0.iter())
                .filter_map(|(dir, n)| (*n > 10).then_some(dir))
        }
    }

    /// A [`Metric`] that optimizes for the _most_ number of steps to a good
    /// terminal node.
    #[derive(Clone, Copy, Debug)]
    pub enum MostSteps {
        /// Still moving through the nodes with the number of steps along the
        /// path so far.
        StillMoving(usize),
        /// Reached a good terminal node with the number of steps it took to get
        /// there.
        Finished(usize),
    }
    impl MostSteps {
        /// Returns the number of steps for either variant.
        pub fn steps(self) -> usize {
            match self {
                MostSteps::StillMoving(n) => n,
                MostSteps::Finished(n) => n,
            }
        }
    }
    impl Default for MostSteps {
        fn default() -> Self {
            Self::StillMoving(0)
        }
    }
    impl Add for MostSteps {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            match self {
                MostSteps::StillMoving(a) => match rhs {
                    MostSteps::StillMoving(b) => Self::StillMoving(a + b),
                    MostSteps::Finished(_) => panic!(),
                },
                MostSteps::Finished(a) => match rhs {
                    MostSteps::StillMoving(_) => panic!(),
                    MostSteps::Finished(b) => Self::Finished(a + b),
                },
            }
        }
    }
    impl From<usize> for MostSteps {
        fn from(value: usize) -> Self {
            Self::StillMoving(value)
        }
    }
    impl Metric for MostSteps {
        fn is_better(&self, other: &Self) -> bool {
            match self {
                MostSteps::StillMoving(a) => match other {
                    MostSteps::StillMoving(b) => a > b,
                    MostSteps::Finished(_) => true,
                },
                MostSteps::Finished(a) => match other {
                    MostSteps::StillMoving(_) => false,
                    MostSteps::Finished(b) => a > b,
                },
            }
        }

        fn successful(self) -> Self {
            Self::Finished(self.steps())
        }
    }

    /// A current vault room.
    ///
    /// This is a [`BestCostTreeNode`] so can be traversed to find an optimal
    /// path through the vault rooms. The type `M` is the [`Metric`] used to
    /// determine the desired path through the vault rooms. This will be the
    /// minimal [`Steps`] for part one and maximal [`MostSteps`] for part two.
    #[derive(Clone, Debug)]
    pub struct CurrentRoom<'a, M> {
        /// The vault associated with the room.
        vault: &'a Vault,
        /// The coordinates of the room.
        point: Point,
        /// The path it took to get to this room from from the
        /// [`Vault::starting_room`].
        path_here: Path,
        /// Phantom data for the metric type `M`.
        _phant: PhantomData<M>,
    }
    impl<M> PartialEq for CurrentRoom<'_, M> {
        fn eq(&self, other: &Self) -> bool {
            self.point == other.point && self.path_here == other.path_here
        }
    }
    impl<M> Eq for CurrentRoom<'_, M> {}
    impl<M> std::hash::Hash for CurrentRoom<'_, M> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.point.hash(state);
            self.path_here.hash(state);
        }
    }
    impl<M: Metric + Copy + Add<Output = M> + Default + From<usize>> BestCostTreeNode
        for CurrentRoom<'_, M>
    {
        type Metric = M;
        type NodeData = Path;

        fn recurse_action(&mut self) -> ApplyNodeAction<BestCostChild<Self>, Self::NodeData> {
            // Have we reached the vault door!?
            if self.point == self.vault.bounding_box.max - Vector::new(1, 1) {
                return ApplyNodeAction::Stop(Some(self.path_here.clone()));
            }

            // What doors are available to us?
            let next_rooms: Vec<_> =
                DoorHash::from(format!("{}{}", self.vault.passcode, self.path_here).as_str())
                    .open_doors()
                    .filter_map(|dir| {
                        let point = self.point + dir.as_vector();
                        self.vault.bounding_box.contains(point).then(|| {
                            BestCostChild::new(
                                Self {
                                    vault: self.vault,
                                    point,
                                    path_here: self.path_here.append(dir),
                                    _phant: PhantomData,
                                },
                                1.into(),
                            )
                        })
                    })
                    .collect();

            if next_rooms.is_empty() {
                // We are in a room with no open doors so are stuck here forever!
                ApplyNodeAction::Stop(None)
            } else {
                ApplyNodeAction::Continue(next_rooms)
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 17,
    name: "Two Steps Forward",
    preprocessor: Some(|input| Ok(Box::new(Vault::new(input.trim())).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Vault>()?
                .starting_room::<Steps>()
                .traverse_tree()?
                .node_data
                .as_string()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(u64::try_from(
                input
                    .expect_data::<Vault>()?
                    .starting_room::<MostSteps>()
                    .traverse_tree()?
                    .cost
                    .steps(),
            )
            .unwrap()
            .into())
        },
    ],
};
