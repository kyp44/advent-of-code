use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "";
            answers = unsigned![123];
        }
        actual_answers = unsigned![123];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::ops::Deref;

    use super::*;
    use aoc::grid::Digit;

    enum LookDirection {
        FromLeft(usize),
        FromRight(usize),
        FromTop(usize),
        FromBottom(usize),
    }

    pub struct TreePatch {
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
        // The index of the furthest visible tree looking from a particular direction.
        // Index is always relative to the grid regardless of look direction.
        fn visible_distance(&self, look_dir: LookDirection) -> usize {
            fn distance(trees: Vec<Digit>) -> Option<usize> {
                trees
                    .into_iter()
                    .adjacent_diff()
                    .position(|d| d <= 0.into())
            }

            let trees = match look_dir {
                LookDirection::FromLeft(r) => self.grid.row_iter(r).copied().collect(),
                LookDirection::FromRight(r) => {
                    let mut v: Vec<_> = self.grid.row_iter(r).copied().collect();
                    v.reverse();
                    v
                }
                LookDirection::FromTop(c) => self.grid.column_iter(c).copied().collect(),
                LookDirection::FromBottom(c) => {
                    let mut v: Vec<_> = self.grid.column_iter(c).copied().collect();
                    v.reverse();
                    v
                }
            };

            iter.deref().copied().adjacent_diff();

            todo!()
        }
        pub fn num_visible(&self) -> u64 {
            0
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 8,
    name: "Treetop Tree House",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let tree_patch = TreePatch::from_str(input.expect_input()?)?;

            // Process
            Ok(tree_patch.num_visible().into())
        },
    ],
};
