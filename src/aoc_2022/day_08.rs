use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "30373
25512
65332
33549
35390";
            answers = unsigned![21, 8];
        }
        actual_answers = unsigned![1782, 474606];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::grid::Digit;
    use strum::{EnumIter, IntoEnumIterator};

    /// A cardinal direction to look relative to a tree in the patch.
    #[derive(Debug, Clone, Copy, EnumIter)]
    enum CardinalDirection {
        /// To the left of the tree.
        Left,
        /// To the right of the tree.
        Right,
        /// Above the tree.
        Up,
        /// Below the tree.
        Down,
    }

    /// A look direction.
    #[derive(Debug, Clone, Copy)]
    enum LookDirection {
        /// In from outside the patch.
        In,
        /// Out from a tree.
        Out,
    }

    /// The patch of trees with which we are concerned.
    pub struct TreePatch {
        /// The grid of tree heights.
        grid: Grid<Digit>,
    }
    impl FromStr for TreePatch {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                grid: Grid::from_str(s)?,
            })
        }
    }
    impl TreePatch {
        /// Returns a list of the trees seen relative to a particular `tree`, in order.
        ///
        /// The direction to look is `cardinal_dir`, but this has a different
        /// meaning depending on the `look_dir`.
        /// If looking in from outside the patch the look from position is in the
        /// `cardinal_dir` from the tree.
        /// As an example, if `cardinal_dir` is [`CardinalDirection::Left`], then we
        /// look right in at the tree from the outside to the left of the patch.
        /// The order of the trees will be reversed whether looking in or looking out.
        /// We include the `tree` itself in the list if `include_tree` is `true`, otherwise
        /// it is not included.
        fn trees_seen(
            &self,
            tree: &GridPoint,
            cardinal_dir: CardinalDirection,
            look_dir: LookDirection,
            include_self: bool,
        ) -> Vec<Digit> {
            // Get the trees seen, including ours
            match cardinal_dir {
                CardinalDirection::Left => {
                    let mut v: Vec<_> = self
                        .grid
                        .row_iter(tree.y)
                        .copied()
                        .take(tree.x + if include_self { 1 } else { 0 })
                        .collect();

                    match look_dir {
                        LookDirection::In => v,
                        LookDirection::Out => {
                            v.reverse();
                            v
                        }
                    }
                }
                CardinalDirection::Right => {
                    let mut v: Vec<_> = self
                        .grid
                        .row_iter(tree.y)
                        .skip(tree.x + if include_self { 0 } else { 1 })
                        .copied()
                        .collect();

                    match look_dir {
                        LookDirection::In => {
                            v.reverse();
                            v
                        }
                        LookDirection::Out => v,
                    }
                }
                CardinalDirection::Up => {
                    let mut v: Vec<_> = self
                        .grid
                        .column_iter(tree.x)
                        .take(tree.y + if include_self { 1 } else { 0 })
                        .copied()
                        .collect();

                    match look_dir {
                        LookDirection::In => v,
                        LookDirection::Out => {
                            v.reverse();
                            v
                        }
                    }
                }
                CardinalDirection::Down => {
                    let mut v: Vec<_> = self
                        .grid
                        .column_iter(tree.x)
                        .skip(tree.y + if include_self { 0 } else { 1 })
                        .copied()
                        .collect();

                    match look_dir {
                        LookDirection::In => {
                            v.reverse();
                            v
                        }
                        LookDirection::Out => v,
                    }
                }
            }
        }

        /// Returns whether a particular `tree` is visible from outside the patch.
        ///
        /// Refer to [`TreePatch::trees_seen`] for a discussion of the meaning of the
        /// `dir`.
        fn visible(&self, dir: CardinalDirection, tree: &GridPoint) -> bool {
            let mut trees = self.trees_seen(tree, dir, LookDirection::In, true);

            let mut visible = true;
            let mut max_height = trees.remove(0);
            for tree in trees {
                if tree > max_height {
                    max_height = tree;
                    visible = true;
                } else {
                    visible = false;
                }
            }

            visible
        }

        /// Calculates the number of trees that are visible from outside the patch
        /// from at least one direction.
        pub fn num_visible(&self) -> u64 {
            self.grid
                .all_points()
                .filter_count(|tree| CardinalDirection::iter().any(|dir| self.visible(dir, tree)))
        }

        /// Returns how many other trees we can see from a potential tree house built
        /// on a particular `tree`.
        ///
        /// This is when looking out in a given `dir` from the potential tree house.
        fn viewing_distance(&self, dir: CardinalDirection, tree: &GridPoint) -> u64 {
            let height = *self.grid.get(tree);
            let trees = self.trees_seen(tree, dir, LookDirection::Out, false);

            let mut count = 0;
            for tree in trees.into_iter() {
                count += 1;
                if tree >= height {
                    break;
                }
            }

            count
        }

        /// Calculates the scenic score for a particular `tree`.
        fn scenic_score(&self, tree: &GridPoint) -> u64 {
            CardinalDirection::iter()
                .map(|dir| self.viewing_distance(dir, tree))
                .product()
        }

        /// Calculates the best possible scenic score, which is where we would like
        /// to build our tree house.
        pub fn best_scenic_score(&self) -> u64 {
            self.grid
                .all_points()
                .map(|tree| self.scenic_score(&tree))
                .max()
                .unwrap_or(0)
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 8,
    name: "Treetop Tree House",
    preprocessor: Some(|input| Ok(Box::new(TreePatch::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<TreePatch>()?.num_visible().into())
        },
        // Part two
        |input| {
            // Process
            Ok(input.expect_data::<TreePatch>()?.best_scenic_score().into())
        },
    ],
};
