use std::str::FromStr;

use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k";
            answers = unsigned![123];
        }
        actual_answers = unsigned![123];
    }
}

/// Contains solution implementation items.
mod solution {
    use std::{iter::Peekable, str::FromStr};

    use super::*;
    use aoc::parse::trim;
    use nom::{
        branch::alt,
        bytes::complete::{tag, take_till},
        character::complete::space1,
        combinator::map,
        sequence::separated_pair,
    };

    #[derive(Debug)]
    enum CommandItem {
        ChangeDir(String),
        List,
    }
    impl Parsable<'_> for CommandItem {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            alt((
                map(
                    separated_pair(tag("cd"), space1::<&str, _>, take_till(char::is_whitespace)),
                    |(_, s)| CommandItem::ChangeDir(s.to_string()),
                ),
                map(trim(false, tag("ls")), |_| CommandItem::List),
            ))(input)
        }
    }

    #[derive(Debug)]
    enum ListingItem {
        Directory(String),
        File(u64),
    }
    impl Parsable<'_> for ListingItem {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            alt((
                map(
                    separated_pair(
                        tag("dir"),
                        space1::<&str, _>,
                        take_till(char::is_whitespace),
                    ),
                    |(_, s)| ListingItem::Directory(s.to_string()),
                ),
                map(
                    separated_pair(
                        nom::character::complete::u64,
                        space1,
                        take_till(char::is_whitespace),
                    ),
                    |(s, _)| ListingItem::File(s),
                ),
            ))(input)
        }
    }

    #[derive(Debug)]
    enum TerminalItem {
        Command(CommandItem),
        Listing(ListingItem),
    }
    impl Parsable<'_> for TerminalItem {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            alt((
                map(
                    separated_pair(tag("$"), space1, CommandItem::parser),
                    |(_, ci)| TerminalItem::Command(ci),
                ),
                map(ListingItem::parser, TerminalItem::Listing),
            ))(input)
        }
    }

    #[derive(Debug)]
    pub enum FileSystem {
        Directory {
            name: String,
            contents: Vec<FileSystem>,
        },
        File(u64),
    }
    impl FromStr for FileSystem {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            fn build(
                item_iter: &mut Peekable<impl Iterator<Item = TerminalItem>>,
                expected_dirs: Vec<String>,
            ) -> AocResult<FileSystem> {
                Err(AocError::Process(
                    if let TerminalItem::Command(CommandItem::ChangeDir(d)) =
                        item_iter.expect_next()?
                    {
                        if expected_dirs.contains(&d) {
                            // The next command should be a list
                            if let TerminalItem::Command(CommandItem::List) =
                                item_iter.expect_next()?
                            {
                                let mut contents = Vec::new();
                                let mut dir_names = Vec::new();

                                // Now go through listed items
                                loop {
                                    if let Some(_) = item_iter.peek().and_then(|x| match x {
                                        TerminalItem::Command(_) => None,
                                        TerminalItem::Listing(_) => Some(()),
                                    }) {
                                        match {
                                            if let TerminalItem::Listing(li) =
                                                item_iter.next().unwrap()
                                            {
                                                li
                                            } else {
                                                panic!();
                                            }
                                        } {
                                            ListingItem::Directory(d) => dir_names.push(d),
                                            ListingItem::File(s) => {
                                                contents.push(FileSystem::File(s))
                                            }
                                        }
                                    } else {
                                        // We are done with the listing
                                        break;
                                    }
                                }

                                return Ok(FileSystem::Directory { name: d, contents });
                            } else {
                                format!("After changing to '{d}'").into()
                            }
                        } else {
                            format!("Directory '{d}' was not previously listed").into()
                        }
                    } else {
                        "The next command is not to change directories".into()
                    },
                ))
            }

            todo!()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 7,
    name: "No Space Left On Device",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let x = FileSystem::from_str(input.expect_input()?)?;
            println!("TODO {x:?}");

            // Process
            Ok(0u64.into())
        },
    ],
};
