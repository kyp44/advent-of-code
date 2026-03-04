use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "Disc #1 has 5 positions; at time=0, it is at position 4.
    Disc #2 has 2 positions; at time=0, it is at position 1.";
            answers = unsigned![5, 85];
        }
        actual_answers = unsigned![376777, 3903937];
    }
}

/// Contains solution implementation items.
mod solution {
    use aoc::parse::trim;
    use bare_metal_modulo::MNum;
    use nom::{
        bytes::complete::tag,
        combinator::map,
        sequence::{preceded, terminated},
    };

    use super::*;

    /// The numeric type for [`Disc`] properties.
    type Num = u8;
    /// The modulo type for solving the modulo system.
    type ModNum = bare_metal_modulo::ModNum<i64>;

    /// A single disc in the [`Sculpture`].
    ///
    /// Can be parsed from text input.
    #[derive(Clone)]
    struct Disc {
        /// The disc number with the top disc being disc 1.
        disc_num: Num,
        /// The number of positions for the disc.
        positions: Num,
        /// The starting position of this disc at time `t = 0`.
        starting_position: Num,
    }
    impl Parsable for Disc {
        type Parsed<'a> = Self;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            use nom::character::complete::u8 as pnum;

            map(
                trim(
                    false,
                    (
                        preceded(tag("Disc #"), pnum),
                        preceded(
                            tag(" has "),
                            terminated(pnum, tag(" positions; at time=0, it is at position ")),
                        ),
                        terminated(pnum, tag(".")),
                    ),
                ),
                |(disc_num, positions, starting_position)| Disc {
                    disc_num,
                    positions,
                    starting_position,
                },
            )
            .parse(input)
        }
    }
    impl Disc {
        /// Generates the modulo equation right hand side for the disc.
        ///
        /// See the notes document for details, in which this value is `x_n`.
        pub fn modulo_equation_rhs(&self) -> ModNum {
            let m = self.positions.into();

            -(ModNum::new(self.starting_position.into(), m) + ModNum::new(self.disc_num.into(), m))
        }
    }

    /// The sculpture.
    ///
    /// Can be parsed from text input.
    #[derive(Clone)]
    pub struct Sculpture {
        /// All the discs, in no particular order.
        discs: Vec<Disc>,
    }
    impl FromStr for Sculpture {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                discs: Disc::gather(s.lines())?,
            })
        }
    }
    impl Sculpture {
        /// Uses the Chinese Remainder Theorem to determine the soonest time
        /// that the capsule can be dropped and will pass through all the discs.
        ///
        /// See the notes document for details.
        pub fn soonest_drop_time(&self) -> AocResult<u64> {
            ModNum::chinese_remainder_system(
                self.discs.iter().map(|disc| disc.modulo_equation_rhs()),
            )
            .map(|mn| mn.a().try_into().unwrap())
            .ok_or(AocError::NoSolution)
        }

        /// Returns a new [`Sculpture`] with the extra disc added for part two.
        pub fn add_extra_disc(&self) -> Self {
            let mut new = self.clone();
            new.discs.push(Disc {
                disc_num: self.discs.iter().map(|disc| disc.disc_num).max().unwrap() + 1,
                positions: 11,
                starting_position: 0,
            });
            new
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 15,
    name: "Timing is Everything",
    preprocessor: Some(|input| Ok(Box::new(Sculpture::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Sculpture>()?
                .soonest_drop_time()?
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Sculpture>()?
                .add_extra_disc()
                .soonest_drop_time()?
                .into())
        },
    ],
};
