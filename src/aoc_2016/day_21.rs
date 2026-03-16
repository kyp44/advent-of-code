use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::prelude_test::*;
    use std::assert_matches;

    solution_tests! {
        // Given scrambling example, not invertible
        example {
            input = "starting password: abcde
scrambled password: decab
swap position 4 with position 0
swap letter d with letter b
reverse positions 0 through 4
rotate left 1 step
move position 1 to position 4
move position 3 to position 0
rotate based on position of letter b
rotate based on position of letter d";
            answers = string!["decab"];
        }
        // Length 8: Fully invertible
        example {
            input = "starting password: abcdefgh
scrambled password: fbdecgha
swap position 4 with position 0
swap letter d with letter b
reverse positions 0 through 4
rotate left 1 step
move position 1 to position 4
move position 3 to position 0
rotate based on position of letter b
rotate based on position of letter d";
            answers = string!["fbdecgha", "abcdefgh"];
        }
        actual_answers = string!["ghfacdbe", "fhgcdaeb"];
    }

    #[test]
    fn rotate_on_letter_inversions() {
        fn execute_inverse(mut pw: Password, idx: usize) -> AocResult<String> {
            InverseOperation::from(Operation::RotateRightOnLetter(pw[idx]))
                .execute(&mut pw)
                .map(|e| e.yielded_item)
        }

        // Length 3: Fully reversible
        let pw = Password::from_str("abc").unwrap();
        for idx in 0..3 {
            assert_matches!(execute_inverse(pw.clone(), idx), Ok(_));
        }

        // Length 5: Partially reversible with the following start -> end index
        // table:
        // 0 -> 1
        // 1 -> 3
        // 2 -> 0
        // 3 -> 2
        // 4 -> 0
        let pw = Password::from_str("abcde").unwrap();
        assert_matches!(execute_inverse(pw.clone(), 0), Err(_));
        assert_matches!(execute_inverse(pw.clone(), 1), Ok(_));
        assert_matches!(execute_inverse(pw.clone(), 2), Ok(_));
        assert_matches!(execute_inverse(pw.clone(), 3), Ok(_));
        assert_matches!(execute_inverse(pw.clone(), 4), Err(_));

        // Length 12: Partially reversible with the following start -> end index
        // table:
        // 0 -> 1
        // 1 -> 3
        // 2 -> 5
        // 3 -> 7
        // 4 -> 10
        // 5 -> 0
        // 6 -> 2
        // 7 -> 4
        // 8 -> 6
        // 9 -> 8
        // 10 -> 10
        // 11 -> 0
        let pw = Password::from_str("abcdefghijkl").unwrap();
        assert_matches!(execute_inverse(pw.clone(), 0), Err(_));
        assert_matches!(execute_inverse(pw.clone(), 1), Ok(_));
        assert_matches!(execute_inverse(pw.clone(), 2), Ok(_));
        assert_matches!(execute_inverse(pw.clone(), 3), Ok(_));
        assert_matches!(execute_inverse(pw.clone(), 4), Ok(_));
        assert_matches!(execute_inverse(pw.clone(), 5), Ok(_));
        assert_matches!(execute_inverse(pw.clone(), 6), Ok(_));
        assert_matches!(execute_inverse(pw.clone(), 7), Ok(_));
        assert_matches!(execute_inverse(pw.clone(), 8), Ok(_));
        assert_matches!(execute_inverse(pw.clone(), 9), Err(_));
        assert_matches!(execute_inverse(pw.clone(), 10), Err(_));
        assert_matches!(execute_inverse(pw.clone(), 11), Err(_));
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::field_line_parser;
    use bare_metal_modulo::{MNum, ModNum};
    use derive_more::From;
    use derive_new::new;
    use itertools::Itertools;
    use nom::{
        Finish, branch::alt, bytes::complete::tag, character::complete::alphanumeric1,
        combinator::map,
    };
    use std::{
        collections::{HashMap, hash_map::Entry},
        ops::Index,
    };

    /// A direction of rotation in a string.
    ///
    /// Can be parsed from text input.
    #[derive(Clone, Copy, Debug)]
    pub enum Direction {
        /// Left.
        Left,
        /// Right.
        Right,
    }
    impl Parsable for Direction {
        type Parsed<'a> = Self;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            alt((
                map(tag("left"), |_| Self::Left),
                map(tag("right"), |_| Self::Right),
            ))
            .parse(input)
        }
    }
    impl std::ops::Neg for Direction {
        type Output = Self;

        fn neg(self) -> Self::Output {
            match self {
                Direction::Left => Self::Right,
                Direction::Right => Self::Left,
            }
        }
    }

    /// An operation used during scrambling.
    ///
    /// Can be parsed from text input.
    #[derive(Clone, Debug)]
    pub enum Operation {
        /// Swaps the position of two indices.
        SwapPositions(usize, usize),
        /// Swaps all pairs of two different letters.
        SwapLetters(char, char),
        /// Rotates the entire string a particular direction some number of
        /// times.
        Rotate(Direction, usize),
        /// Rotates right based on the index of a letter using the particular
        /// rules.
        RotateRightOnLetter(char),
        /// Reverses a substring between (and including) two indices.
        ReverseBetweenPositions(usize, usize),
        /// Moves the letter at an index so that it will be at a different
        /// index.
        MovePositions(usize, usize),
    }
    impl Parsable for Operation {
        type Parsed<'a> = Self;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            use nom::character::complete::{anychar, usize as pnum};

            alt((
                map(
                    (tag("swap position "), pnum, tag(" with position "), pnum),
                    |(_, x, _, y)| Self::SwapPositions(x, y),
                ),
                map(
                    (tag("swap letter "), anychar, tag(" with letter "), anychar),
                    |(_, x, _, y)| Self::SwapLetters(x, y),
                ),
                map(
                    (
                        tag("rotate "),
                        Direction::parser,
                        tag(" "),
                        pnum,
                        tag(" step"),
                    ),
                    |(_, dir, _, n, _)| Self::Rotate(dir, n),
                ),
                map(
                    (tag("rotate based on position of letter "), anychar),
                    |(_, c)| Self::RotateRightOnLetter(c),
                ),
                map(
                    (tag("reverse positions "), pnum, tag(" through "), pnum),
                    |(_, x, _, y)| Self::ReverseBetweenPositions(x, y),
                ),
                map(
                    (tag("move position "), pnum, tag(" to position "), pnum),
                    |(_, x, _, y)| Self::MovePositions(x, y),
                ),
            ))
            .parse(input)
        }
    }
    impl Instruction for Operation {
        type Registers = Password;
        type YieldItem = String;
        type Err = AocError;

        fn execute(
            &self,
            registers: &mut Self::Registers,
        ) -> Result<Executed<Self::YieldItem>, Self::Err> {
            match self {
                Operation::SwapPositions(ix, iy) => registers.characters.swap(*ix, *iy),
                Operation::SwapLetters(cx, cy) => {
                    let ixs: Vec<_> = registers.characters.iter().positions(|c| c == cx).collect();
                    let iys: Vec<_> = registers.characters.iter().positions(|c| c == cy).collect();
                    for (ix, iy) in ixs.into_iter().zip(iys) {
                        registers.characters[ix] = *cy;
                        registers.characters[iy] = *cx;
                    }
                }
                Operation::Rotate(dir, n) => match dir {
                    Direction::Left => registers.characters.rotate_left(*n),
                    Direction::Right => registers.characters.rotate_right(*n),
                },
                Operation::RotateRightOnLetter(cx) => {
                    let ix = registers
                        .characters
                        .iter()
                        .position(|c| c == cx)
                        .ok_or_else(|| {
                            AocError::Process(
                        format!(
                            "The character '{cx}' does not appear in the password string '{}'",
                            registers.as_string()
                        )
                        .into(),
                    )
                        })?;
                    let nr =
                        Password::rotation_for_index(registers.characters.len(), ix).num_rotations;

                    registers.characters.rotate_right(nr);
                }
                Operation::ReverseBetweenPositions(ix, iy) => {
                    registers.characters[(*ix).min(*iy)..=(*ix).max(*iy)].reverse()
                }
                Operation::MovePositions(ix, iy) => {
                    let x = registers.characters.remove(*ix);
                    registers.characters.insert(*iy, x);
                }
            }

            Ok(Executed::new(registers.as_string(), None))
        }
    }

    /// An inverse operation used in unscrambling a scrambled password.
    ///
    /// All operations are always invertible except
    /// [`Operation::RotateRightOnLetter`]. In particular, for password lengths
    /// other than 3 and 8, some distinct character indices map to the
    /// same index after the rotation. In practice, inverting this operation
    /// will fail only if the character is at one of these ambiguous
    /// indices.
    #[derive(From)]
    pub struct InverseOperation {
        /// The original operation.
        operation: Operation,
    }
    impl Instruction for InverseOperation {
        type Registers = Password;
        type YieldItem = String;
        type Err = AocError;

        fn execute(
            &self,
            registers: &mut Self::Registers,
        ) -> Result<Executed<Self::YieldItem>, Self::Err> {
            match self.operation {
                Operation::Rotate(dir, n) => Operation::Rotate(-dir, n).execute(registers),
                Operation::RotateRightOnLetter(cx) => {
                    let ix = registers.characters.iter().position(|c| *c == cx).unwrap();
                    let nr = registers.reverse_letter_rotations.get(&ix).ok_or_else(|| {
                        AocError::Process(
                            format!(
                                "The rotate right on letter operation could not be unambiguously reversed for password '{}' and letter '{}' at index {}",
                                registers.as_string(),
                                cx,
                                ix,
                            )
                            .into(),
                        )
                    })?;

                    registers.characters.rotate_left(*nr);
                    Ok(Executed::new(registers.as_string(), None))
                }
                Operation::MovePositions(ix, iy) => {
                    Operation::MovePositions(iy, ix).execute(registers)
                }
                _ => {
                    // The rest of the operations are their own inverse
                    self.operation.execute(registers)
                }
            }
        }
    }

    /// Details required to perform the forward and inverse operations when
    /// using [`Operation::RotateRightOnLetter`].
    ///
    /// This should be obtained by calling [`Password::rotation_for_index`].
    #[derive(Debug)]
    pub struct LetterRotation {
        /// The number of rotations right.
        pub num_rotations: usize,
        /// Where the original index will end up after the rotations.
        pub end_index: usize,
    }

    /// A password string in which every character appears at most once, which
    /// is necessary for operations to be invertible.
    #[derive(Clone)]
    pub struct Password {
        /// The characters in the string, easier to work with than a string
        /// type.
        characters: Vec<char>,
        /// The reverse letter rotation table when using the
        /// [`InverseOperation`] for [`Operation::RotateRightOnLetter`].
        ///
        /// This maps the character end index (in the forward [`Operation`]) to
        /// the number of rotations left needed to invert the operation. Only
        /// end indices for which the original indices are unambiguous will be
        /// keys.
        reverse_letter_rotations: HashMap<usize, usize>,
    }
    impl FromStr for Password {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // Verify the that letters are unique
            let characters: Vec<char> = s.chars().collect();
            characters.iter().all_unique().ok_or_else(|| {
                AocError::InvalidInput(
                    format!("The password '{s}' does not have all unique letters").into(),
                )
            })?;

            // Only map entrees where there are no duplicate end indices
            let mut reverse_letter_rotations = HashMap::new();
            let mut bad_end_idxs = Vec::new();
            for letter_rot in (0..s.len()).map(|idx| Password::rotation_for_index(s.len(), idx)) {
                if let Entry::Vacant(vacant) = reverse_letter_rotations.entry(letter_rot.end_index)
                {
                    vacant.insert(letter_rot.num_rotations);
                } else {
                    bad_end_idxs.push(letter_rot.end_index);
                }
            }
            for end_idx in bad_end_idxs.iter() {
                reverse_letter_rotations.remove(end_idx);
            }

            Ok(Self {
                characters,
                reverse_letter_rotations,
            })
        }
    }
    impl Index<usize> for Password {
        type Output = char;

        fn index(&self, index: usize) -> &Self::Output {
            &self.characters[index]
        }
    }
    impl Password {
        /// Calculates and returns the letter rotations for a password of length
        /// `len` and a character at index `idx` for the
        /// [`Operation::RotateRightOnLetter`] operation.
        pub fn rotation_for_index(len: usize, idx: usize) -> LetterRotation {
            let idx = ModNum::new(idx, len);

            let nr = if idx < 4 { idx + 1 } else { idx + 2 };

            LetterRotation {
                num_rotations: nr.a(),
                end_index: (idx + nr).a(),
            }
        }

        /// Returns the password as a string.
        pub fn as_string(&self) -> String {
            self.characters.iter().collect()
        }
    }

    /// A program that scrambles or unscrambles a password for the operation
    /// `I`.
    #[derive(new)]
    pub struct PasswordProgram<I> {
        /// The underlying program.
        program: Program<I>,
    }
    impl<I: Instruction> PasswordProgram<I>
    where
        I::Registers: FromStr<Err = AocError>,
    {
        /// Executes the (un)scrambling program.
        pub fn execute(&self, password: &str) -> AocResult<I::YieldItem>
        where
            I::Registers: FromStr<Err = AocError>,
            I::Err: Into<AocError>,
        {
            self.program
                .execute(password.parse()?)
                .map_err(Into::into)?
                .last_yielded
                .ok_or(AocError::NoSolution)
        }
    }
    impl PasswordProgram<Operation> {
        /// Inverts and returns the program by inverting each [`Operation`]
        /// instruction and reversing their order.
        pub fn invert(&self) -> PasswordProgram<InverseOperation> {
            let mut instructions: Vec<InverseOperation> = self
                .program
                .instructions()
                .iter()
                .map(|inst| inst.clone().into())
                .collect();
            instructions.reverse();

            PasswordProgram::new(Program::new(instructions))
        }
    }

    /// The overall problem definition for both parts.
    ///
    /// Can be parsed from text input.
    pub struct PasswordProblem {
        /// The starting password to be scrambled.
        pub starting_password: String,
        /// The scrambled password to unscramble.
        pub scrambled_password: String,
        /// The scrambling program.
        pub program: PasswordProgram<Operation>,
    }
    impl FromStr for PasswordProblem {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let (program_str, (stp, scp)) = (
                field_line_parser("starting password:", alphanumeric1::<_, NomParseError>),
                field_line_parser("scrambled password:", alphanumeric1),
            )
                .parse(s)
                .finish()?;

            Ok(Self {
                starting_password: stp.into(),
                scrambled_password: scp.into(),
                program: PasswordProgram::new(Program::parse(program_str)?),
            })
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 21,
    name: "Scrambled Letters and Hash",
    preprocessor: Some(|input| Ok(Box::new(PasswordProblem::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            let problem = input.expect_data::<PasswordProblem>()?;
            Ok(problem
                .program
                .execute(problem.starting_password.as_str())?
                .into())
        },
        // Part two
        |input| {
            // Process
            let problem = input.expect_data::<PasswordProblem>()?;
            Ok(problem
                .program
                .invert()
                .execute(problem.scrambled_password.as_str())?
                .into())
        },
    ],
};
