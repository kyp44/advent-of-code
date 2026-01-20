use aoc::prelude::*;
use std::str::FromStr;

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
            answers = unsigned![95437, 24933642];
        }
        actual_answers = unsigned![1583951, 214171];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use enum_as_inner::EnumAsInner;
    use nom::{
        branch::alt,
        bytes::complete::{tag, take_till},
        character::complete::space1,
        combinator::map,
        sequence::separated_pair,
    };
    use std::iter::{FusedIterator, Peekable};
    use takeable::Takeable;

    /// A command item from the terminal output.
    #[derive(Debug, EnumAsInner)]
    enum CommandItem {
        /// Change the current directory with the directory name to which to change.
        ChangeDir(String),
        /// List the contents of the current directory.
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
            ))
            .parse(input)
        }
    }

    /// A listing output item from the terminal output.
    #[derive(Debug)]
    enum ListingItem {
        /// A sub directory with its name.
        Directory(String),
        /// A file with its size.
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
            ))
            .parse(input)
        }
    }

    /// An item parsed from the terminal output.
    #[derive(Debug, EnumAsInner)]
    enum TerminalItem {
        /// A command item.
        Command(CommandItem),
        /// A listing output item.
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
            ))
            .parse(input)
        }
    }

    /// A recursive file system item.
    #[derive(Debug, EnumAsInner)]
    enum FileSystem {
        /// This is a directory with its [`Directory`].
        Directory(Directory),
        /// This is a file with its size.
        File(u64),
    }

    /// A directory in the file system.
    #[derive(Debug)]
    pub struct Directory {
        /// The name of the directory.
        pub name: String,
        /// The contents of the directory.
        contents: Vec<FileSystem>,
    }
    impl FromStr for Directory {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            /// Builds a directory from the parsed terminal output.
            ///
            /// This is a recursive internal function of [`Directory::from_str`].
            fn build(
                item_iter: &mut Peekable<impl Iterator<Item = TerminalItem>>,
            ) -> AocResult<Directory> {
                // First, we expect a change directory command
                let dir_name = item_iter
                    .expect_next()?
                    .into_command()
                    .ok()
                    .and_then(|ci| ci.into_change_dir().ok())
                    .ok_or(AocError::Process(
                        "The next command is not to change directories".into(),
                    ))?;

                // The next command should be a list
                if !item_iter
                    .expect_next()?
                    .into_command()
                    .ok()
                    .map(|ci| ci.is_list())
                    .unwrap_or(false)
                {
                    return Err(AocError::Process(
                        format!("After changing to '{dir_name}', the next command was not a list")
                            .into(),
                    ));
                }

                let mut contents = Vec::new();
                let mut dir_names = Vec::new();

                // Now go through listed items
                loop {
                    // We are done if no more items are listed
                    if !item_iter.peek().map(|x| x.is_listing()).unwrap_or(false) {
                        break;
                    }

                    match item_iter.next().unwrap().into_listing().unwrap() {
                        ListingItem::Directory(d) => dir_names.push(d),
                        ListingItem::File(s) => contents.push(FileSystem::File(s)),
                    }
                }

                // Now that we have everything, we expect that each directory to be listed recursively
                loop {
                    if dir_names.is_empty() {
                        break;
                    }

                    let sub_dir = build(item_iter)?;

                    // Remove this sub directory from our list
                    dir_names.swap_remove(
                        dir_names
                            .iter()
                            .position(|d| *d == sub_dir.name)
                            .ok_or_else(|| {
                                AocError::Process(
                                    format!(
                                        "Directory '{}' was not previously listed",
                                        sub_dir.name
                                    )
                                    .into(),
                                )
                            })?,
                    );

                    // Now add the sub directory to our contents
                    contents.push(FileSystem::Directory(sub_dir));
                }

                // Finally, we should have a cd back to the parent unless we are at the very end
                if !item_iter
                    .next()
                    .map(|ti| {
                        ti.into_command()
                            .ok()
                            .and_then(|ci| ci.into_change_dir().ok().map(|d| d == ".."))
                            .unwrap_or(false)
                    })
                    .unwrap_or(true)
                {
                    return Err(AocError::Process(
                        "The sub directory listing was not followed by a change to the parent"
                            .into(),
                    ));
                }

                Ok(Directory {
                    name: dir_name,
                    contents,
                })
            }

            let mut iterm_iter = TerminalItem::gather(s.lines())?.into_iter().peekable();
            let root_dir = build(&mut iterm_iter)?;

            if root_dir.name != "/" {
                return Err(AocError::Process(
                    "The initial listed directory is not root!".into(),
                ));
            }

            Ok(root_dir)
        }
    }
    impl Directory {
        /// Returns a recursive [`Iterator`] over all directories, starting with this one.
        pub fn all_directories(&self) -> DirectoryTraversal<'_> {
            DirectoryTraversal {
                current: Takeable::new(self),
                contents: self.contents.iter(),
                sub_iter: None,
            }
        }

        /// Returns the total size contained within this directory recursively.
        pub fn size(&self) -> u64 {
            self.contents
                .iter()
                .map(|fs| match fs {
                    FileSystem::Directory(d) => d.size(),
                    FileSystem::File(s) => *s,
                })
                .sum()
        }
    }

    /// A recursive [`Iterator`] over the directories within a [`Directory`].
    ///
    /// This should only ever be returned from [`Directory::all_directories`].
    pub struct DirectoryTraversal<'a> {
        /// The current [`Directory`], indicating whether it has been emitted yet.
        current: Takeable<&'a Directory>,
        /// The contents of the current directory as an [`Iterator`].
        contents: std::slice::Iter<'a, FileSystem>,
        /// The [`Iterator`] over the current sub-directory.
        ///
        /// This will be set only if we are currently iterating over a sub-directory.
        sub_iter: Option<Box<DirectoryTraversal<'a>>>,
    }
    impl<'a> Iterator for DirectoryTraversal<'a> {
        type Item = &'a Directory;

        fn next(&mut self) -> Option<Self::Item> {
            // Return ourself if have not already done so
            if self.current.is_usable() {
                return Some(self.current.take());
            }

            if let Some(iter) = self.sub_iter.as_deref_mut()
                && let Some(fs) = iter.next()
            {
                // Return the next sub-directory item if there is any
                Some(fs)
            } else {
                // Otherwise, we need to advance to the next item within this directory
                self.contents.find_map(|fs| {
                    match fs {
                        FileSystem::Directory(d) => {
                            // Create iterator over the contents
                            let mut sub = d.all_directories();
                            let next = sub.next().unwrap();
                            self.sub_iter = Some(sub.into());
                            Some(next)
                        }
                        FileSystem::File(_) => None,
                    }
                })
            }
        }
    }
    impl FusedIterator for DirectoryTraversal<'_> {}
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 7,
    name: "No Space Left On Device",
    preprocessor: Some(|input| Ok(Box::new(Directory::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<Directory>()?
                    .all_directories()
                    .filter_map(|d| {
                        let ds = d.size();

                        if ds <= 100000 { Some(ds) } else { None }
                    })
                    .sum(),
            ))
        },
        // Part two
        |input| {
            // Generation
            let root = input.expect_data::<Directory>()?;

            // Process
            /// The total disk space of the root file system.
            const TOTAL_SPACE: u64 = 70000000;
            /// The minimum space we need to perform the update.
            const NEEDED_SPACE: u64 = 30000000;

            let free_space = TOTAL_SPACE - root.size();
            let need_to_free = NEEDED_SPACE - free_space;

            Ok(Answer::Unsigned(
                root.all_directories()
                    .filter_map(|d| {
                        let ds = d.size();

                        if ds >= need_to_free { Some(ds) } else { None }
                    })
                    .min()
                    .ok_or(AocError::NoSolution)?,
            ))
        },
    ],
};
