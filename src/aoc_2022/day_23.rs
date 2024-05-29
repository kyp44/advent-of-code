use aoc::prelude::*;
use gat_lending_iterator::LendingIterator;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = ".....
..##.
..#..
.....
..##.
.....";
            answers = unsigned![123];
        }
        example {
            input = "....#..
..###.#
#...#.#
.#...##
#.###..
##.#.##
.#..#..";
            answers = unsigned![110];
        }
        actual_answers = unsigned![123];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::{
        collections::{HashSet, VecDeque},
        sync::LazyLock,
    };

    use super::*;
    use aoc::grid::StdBool;
    use euclid::default::{Point2D, Vector2D};
    use gat_lending_iterator::LendingIterator;
    use itertools::Itertools;
    use multiset::HashMultiSet;
    use strum::{EnumIter, IntoEnumIterator};

    #[derive(Clone, Copy, Debug, Default, EnumIter)]
    enum Move {
        #[default]
        North,
        South,
        West,
        East,
    }
    impl Move {
        pub fn as_vector(&self) -> Vector2D<isize> {
            match self {
                Move::North => -ElfVector::unit_y(),
                Move::South => ElfVector::unit_y(),
                Move::West => -ElfVector::unit_x(),
                Move::East => ElfVector::unit_x(),
            }
        }

        pub fn checked_position_displacements(&self) -> &'static [ElfVector] {
            static NORTH_VECS: LazyLock<[ElfVector; 3]> = LazyLock::new(|| {
                [
                    Move::North.as_vector() + Move::West.as_vector(),
                    Move::North.as_vector(),
                    Move::North.as_vector() + Move::East.as_vector(),
                ]
            });
            static SOUTH_VECS: LazyLock<[ElfVector; 3]> = LazyLock::new(|| {
                [
                    Move::South.as_vector() + Move::West.as_vector(),
                    Move::South.as_vector(),
                    Move::South.as_vector() + Move::East.as_vector(),
                ]
            });
            static WEST_VECS: LazyLock<[ElfVector; 3]> = LazyLock::new(|| {
                [
                    Move::West.as_vector() + Move::South.as_vector(),
                    Move::West.as_vector(),
                    Move::West.as_vector() + Move::North.as_vector(),
                ]
            });
            static EAST_VECS: LazyLock<[ElfVector; 3]> = LazyLock::new(|| {
                [
                    Move::East.as_vector() + Move::South.as_vector(),
                    Move::East.as_vector(),
                    Move::East.as_vector() + Move::North.as_vector(),
                ]
            });

            match self {
                Move::North => &*NORTH_VECS,
                Move::South => &*SOUTH_VECS,
                Move::West => &*WEST_VECS,
                Move::East => &*EAST_VECS,
            }
        }
    }

    type ElfPoint = Point2D<isize>;
    type ElfVector = Vector2D<isize>;

    #[derive(Debug)]
    struct Elf {
        position: ElfPoint,
        proposed_move: Option<Move>,
    }
    impl Elf {
        pub fn proposed_position(&self) -> Option<ElfPoint> {
            self.proposed_move.map(|m| self.position + m.as_vector())
        }
    }

    pub struct Grove {
        move_order: VecDeque<Move>,
        elves: Box<[Elf]>,
    }
    impl FromStr for Grove {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // Parse the boolean grid
            let grid = Grid::<StdBool>::from_str(s)?;

            // Create the elves with bogus proposed moves
            let elves = grid
                .as_coordinates()
                .into_iter()
                .map(|p| Elf {
                    position: p.cast_unit().try_cast().unwrap(),
                    proposed_move: None,
                })
                .collect();

            let grove = Self {
                move_order: Move::iter().collect(),
                elves,
            };

            Ok(grove)
        }
    }
    impl std::fmt::Debug for Grove {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let positions = self.as_coordinates().map(|p| p.cast_unit()).collect_vec();

            write!(
                f,
                "{:?}",
                Grid::<StdBool>::from_coordinates(positions.iter())
            )
        }
    }
    impl Grove {
        fn as_coordinates<'a>(&'a self) -> impl Iterator<Item = ElfPoint> + 'a {
            self.elves.iter().map(|e| e.position)
        }

        // Uses current `move_order` and does not alter it, making this yield the same results
        // if called more than once.
        fn propose_moves(&mut self) {
            let positions: HashSet<_> = self.as_coordinates().collect();

            for elf in self.elves.iter_mut() {
                // If there are no neighboring elves, do nothing
                elf.proposed_move = if elf
                    .position
                    .all_neighbor_points(true, false)
                    .any(|p| positions.contains(&p))
                {
                    let mut new_move = None;

                    for muv in self.move_order.iter() {
                        if muv
                            .checked_position_displacements()
                            .iter()
                            .all(|d| !positions.contains(&(elf.position + *d)))
                        {
                            // This is the move we will propose
                            new_move = Some(*muv);
                            break;
                        }
                    }
                    if new_move.is_none() {
                        panic!("The elf at {:?} has no move to propose", elf.position);
                    }

                    new_move
                } else {
                    None
                };
            }

            println!("TODO proposed moves:");
            for elf in self.elves.iter() {
                println!("{elf:?}");
            }
        }
    }
    impl LendingIterator for Grove {
        type Item<'a> = &'a Self
        where
            Self: 'a;

        fn next(&mut self) -> Option<Self::Item<'_>> {
            // Have every elf propose a move
            self.propose_moves();

            // Now execute the proposed moves
            let new_positions =
                HashMultiSet::from_iter(self.elves.iter().filter_map(|e| e.proposed_position()));
            for elf in self.elves.iter_mut().filter(|e| e.proposed_move.is_some()) {
                // Only move elves for which they are the only one moving to their new position
                let new_position = elf.proposed_position().unwrap();
                if new_positions.count_of(&new_position) == 1 {
                    elf.position = new_position;
                }
            }

            // Change the move order for next time
            self.move_order.rotate_left(1);

            // Check whether we are done, which is when all elves propose no move
            if self.elves.iter().all(|e| e.proposed_move.is_none()) {
                None
            } else {
                Some(self)
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 23,
    name: "Unstable Diffusion",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let mut grove = Grove::from_str(input.expect_text()?)?;

            println!("TODO initial:\n{grove:?}");
            grove.take(10).for_each(|g| {
                println!("\nTODO\n{g:?}");
            });

            // Process
            Ok(123u64.into())
        },
    ],
};
