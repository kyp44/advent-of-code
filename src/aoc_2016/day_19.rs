use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "5";
            answers = unsigned![3, 2];
        }
        example {
            input = "20";
            answers = unsigned![9, 13];
        }
        actual_answers = unsigned![1830117, 1417887];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use bare_metal_modulo::{MNum, ModNum};
    use num::Integer;

    /// A single elf that is participating in the gift game.
    #[derive(Clone, Debug)]
    pub struct Elf {
        /// The number of the elf, starting at `1`.
        pub elf_num: u32,
        /// The number of presents the elf has.
        pub presents: u32,
    }
    impl Elf {
        /// Creates a new elf with the given `elf_num` and one present.
        pub fn new(elf_num: u32) -> Self {
            assert!(elf_num > 0);
            Self {
                elf_num,
                presents: 1,
            }
        }
    }

    /// The elf circle for the gift game.
    #[derive(Clone)]
    pub struct ElfCircle {
        /// All the elves, with `None` indicating that the elf at that position
        /// has left the circle.
        elves: Vec<Option<Elf>>,
        /// The number of elves remaining, that is the number of `elves` that
        /// have a `Some` variant.
        num_elves_left: usize,
    }
    impl FromStr for ElfCircle {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let s = s.trim();
            s.parse::<u32>()
                .map_err(|_| {
                    AocError::InvalidInput(format!("'{s}' is not a valid number of elves").into())
                })
                .map(|n| Self {
                    elves: (0..n).map(|idx| Some(Elf::new(idx + 1))).collect(),
                    num_elves_left: n.try_into().unwrap(),
                })
        }
    }
    impl ElfCircle {
        /// Plays the game starting from this circle state until only one elf
        /// has all the presents.
        ///
        /// Returns the [`Elf`] that ends up with all the presents.
        pub fn play_game<S: StealMethod>(&self) -> Elf {
            let mut circle = self.clone();

            loop {
                let mut steal_method = S::default();
                for elf_idx in 0..circle.elves.len() {
                    if circle.elves[elf_idx].is_some() {
                        match steal_method.elf_to_steal_from(&circle, elf_idx) {
                            Some(victim_idx) => circle.steal(elf_idx, victim_idx),
                            None => panic!("No elves form whom to steal!"),
                        }
                    }
                }

                // Remove all elves that were eliminated, for efficiency
                circle.elves.retain(|e| e.is_some());

                // If there is only one elf remaining, we are done.
                if circle.elves.len() == 1 {
                    break circle.elves.into_iter().next().unwrap().unwrap();
                }
            }
        }

        /// Returns the index of the elf to the left in the circle
        /// `offset` number of elves from the `base_idx`.
        ///
        /// `None` is returned if there is no such elf or if the only remaining
        /// elf is at `base_idx`.
        fn index_offset(&self, base_idx: usize, offset: usize) -> Option<usize> {
            let base_idx = ModNum::new(base_idx, self.elves.len());
            let mut target_idx = base_idx;

            for _ in 0..offset {
                target_idx += 1;

                while self.elves[target_idx.a()].is_none() {
                    target_idx += 1;
                }
            }

            (base_idx != target_idx).then_some(target_idx.a())
        }

        /// Steals all the presents from the elf at the `victim_idx` and gives
        /// them to the elf at the `elf_idx`, then removes the victim elf from
        /// the circle.
        ///
        /// This assumes that both elves have not already left the circle, and
        /// will panic if they either has.
        fn steal(&mut self, elf_idx: usize, victim_idx: usize) {
            self.elves[elf_idx].as_mut().unwrap().presents +=
                self.elves[victim_idx].as_ref().unwrap().presents;
            self.elves[victim_idx] = None;
            self.num_elves_left -= 1;
        }
    }

    /// Implementors provide an elf from whom to steal presents.
    pub trait StealMethod: Default {
        /// Returns the index in the `circle` of the elf from whom the elf at
        /// `elf_idx` will steal presents, or `None` if no there is no elf from
        /// whom to steal.
        fn elf_to_steal_from(&mut self, circle: &ElfCircle, elf_idx: usize) -> Option<usize>;
    }

    /// Steals presents from the next elf to the left who is still in the
    /// circle for part one.
    #[derive(Default)]
    pub struct StealLeft;
    impl StealMethod for StealLeft {
        fn elf_to_steal_from(&mut self, circle: &ElfCircle, elf_idx: usize) -> Option<usize> {
            circle.index_offset(elf_idx, 1)
        }
    }

    /// Steals presents from the elf sitting across in the circle.
    #[derive(Default)]
    pub struct StealAcross {
        /// The index of the elf currently sitting across, or `None` if this has
        /// not been determined yet.
        across_idx: Option<usize>,
    }
    impl StealMethod for StealAcross {
        fn elf_to_steal_from(&mut self, circle: &ElfCircle, elf_idx: usize) -> Option<usize> {
            let victim_idx = match self.across_idx.as_ref() {
                Some(across_idx) => circle.index_offset(
                    *across_idx,
                    if circle.num_elves_left.is_even() {
                        2
                    } else {
                        1
                    },
                ),
                None => circle.index_offset(elf_idx, circle.num_elves_left / 2),
            };

            self.across_idx = victim_idx;
            victim_idx
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 19,
    name: "An Elephant Named Joseph",
    preprocessor: Some(|input| Ok(Box::new(ElfCircle::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(u64::from(
                input
                    .expect_data::<ElfCircle>()?
                    .play_game::<StealLeft>()
                    .elf_num,
            )
            .into())
        },
        // Part two
        |input| {
            // Process
            Ok(u64::from(
                input
                    .expect_data::<ElfCircle>()?
                    .play_game::<StealAcross>()
                    .elf_num,
            )
            .into())
        },
    ],
};
