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
        actual_answers = unsigned![1782];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::grid::Digit;
    use strum::IntoEnumIterator;
    use strum_macros::EnumIter;

    #[derive(Debug, Clone, Copy, EnumIter)]
    enum LookDirection {
        FromLeft,
        FromRight,
        FromTop,
        FromBottom,
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
        fn visible(&self, look_dir: LookDirection, tree: &GridPoint) -> bool {
            struct Line {
                trees: Vec<Digit>,
                idx: usize,
            }

            let size = self.grid.size();
            let line = match look_dir {
                LookDirection::FromLeft => Line {
                    trees: self.grid.row_iter(tree.y).copied().collect(),
                    idx: tree.x,
                },
                LookDirection::FromRight => {
                    let mut trees: Vec<_> = self.grid.row_iter(tree.y).copied().collect();
                    trees.reverse();

                    Line {
                        trees,
                        idx: size.x - 1 - tree.x,
                    }
                }
                LookDirection::FromTop => Line {
                    trees: self.grid.column_iter(tree.x).copied().collect(),
                    idx: tree.y,
                },
                LookDirection::FromBottom => {
                    let mut trees: Vec<_> = self.grid.column_iter(tree.x).copied().collect();
                    trees.reverse();

                    Line {
                        trees,
                        idx: size.y - 1 - tree.y,
                    }
                }
            };

            let mut visible = true;
            let mut max_height = line.trees[0];
            for tree in line.trees.into_iter().skip(1).take(line.idx) {
                if tree > max_height {
                    max_height = tree;
                    visible = true;
                } else {
                    visible = false;
                }
            }

            visible
        }

        pub fn num_visible(&self) -> u64 {
            self.grid.all_points().filter_count(|tree| {
                LookDirection::iter().any(|look_dir| self.visible(look_dir, tree))
            })
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
            Ok(input.expect_data::<TreePatch>()?.num_visible().into())
        },
    ],
};
