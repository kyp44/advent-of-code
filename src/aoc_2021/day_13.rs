use crate::aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
        vec![Unsigned(592), Unsigned(94)],
    "6,10
0,14
9,10
0,3
10,4
4,11
6,0
6,12
4,1
0,13
10,12
3,4
3,0
8,4
1,10
2,14
8,10
9,0

fold along y=7
fold along x=5",
        vec![17u64, 16].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use crate::aoc::parse::trim;
    use cgmath::Vector2;
    use nom::{
        bytes::complete::tag,
        character::complete::{multispace1, one_of},
        combinator::map,
        sequence::{preceded, separated_pair},
    };
    use shrinkwraprs::Shrinkwrap;
    use std::{collections::HashSet, fmt::Debug, rc::Rc};

    /// A dot location on the transparent page, which can be parsed from text input.
    #[derive(Shrinkwrap, PartialEq, Eq, Hash, Clone)]
    struct Dot(Vector2<isize>);
    impl Parseable<'_> for Dot {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_pair(
                    nom::character::complete::i32,
                    trim(false, tag(",")),
                    nom::character::complete::i32,
                ),
                |(x, y)| Self(Vector2::new(x.try_into().unwrap(), y.try_into().unwrap())),
            )(input)
        }
    }
    impl Dot {
        /// Create a new dot based on its coordinates on the page.
        fn new(x: isize, y: isize) -> Self {
            Self(Vector2::new(x, y))
        }
    }

    /// A transparent page, which can be parsed from text input.
    #[derive(Clone)]
    pub struct Page {
        /// The set of dots on the page.
        dots: HashSet<Dot>,
    }
    impl FromStr for Page {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Page {
                dots: Dot::gather(s.lines())?.into_iter().collect(),
            })
        }
    }
    impl Debug for Page {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let grid = Grid::from_coordinates(self.dots.iter().map(|d| **d));
            write!(f, "{grid:?}")
        }
    }
    impl Page {
        /// Fold the page and return the new folded page.
        fn fold(&self, fold: &Fold) -> Self {
            let mut dots = HashSet::new();
            match fold {
                Fold::Vertical(fx) => {
                    for dot in self.dots.iter().map(Dot::as_ref) {
                        dots.insert(Dot::new(
                            if dot.x <= *fx { dot.x } else { 2 * fx - dot.x },
                            dot.y,
                        ));
                    }
                }
                Fold::Horizontal(fy) => {
                    for dot in self.dots.iter().map(Dot::as_ref) {
                        dots.insert(Dot::new(
                            dot.x,
                            if dot.y <= *fy { dot.y } else { 2 * fy - dot.y },
                        ));
                    }
                }
            }
            Self { dots }
        }

        /// Return the number of dots on the page.
        pub fn num_dots(&self) -> usize {
            self.dots.len()
        }
    }

    /// A way and location to fold a page, which can be parsed from text input.
    #[derive(Clone, Copy)]
    enum Fold {
        /// Vertical fold at the `x` coordinate of the vertical line.
        Vertical(isize),
        /// Horizontal fold at the `y` coordinate of the horizontal line.
        Horizontal(isize),
    }
    impl Parseable<'_> for Fold {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                preceded(
                    preceded(tag("fold along"), multispace1),
                    separated_pair(one_of("xy"), tag("="), nom::character::complete::u32),
                ),
                |(dir, val)| {
                    let val = val.try_into().unwrap();
                    match dir {
                        'x' => Self::Vertical(val),
                        _ => Self::Horizontal(val),
                    }
                },
            )(input)
        }
    }

    /// The problem definition, which can be parsed from text input.
    pub struct Problem {
        /// The initial page from the manual.
        page: Page,
        /// The ordered list of folds to arrive at the final pattern of dots.
        folds: Box<[Fold]>,
    }
    impl FromStr for Problem {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let sections = s.sections(2)?;

            Ok(Self {
                page: Page::from_str(sections[0])?,
                folds: Fold::gather(sections[1].lines())?.into_boxed_slice(),
            })
        }
    }
    impl Problem {
        /// Apply the complete sequence of folds.
        ///
        /// Returns an [Iterator] of pages after each fold is made.
        pub fn apply_folds(&self) -> FoldedPages {
            FoldedPages {
                page: Rc::new(self.page.clone()),
                folds: self.folds.iter(),
            }
        }
    }

    /// [Iterator] over the pages after each fold.
    pub struct FoldedPages<'a> {
        /// The current page.
        page: Rc<Page>,
        /// [Iterator] over the actual folds.
        folds: std::slice::Iter<'a, Fold>,
    }
    impl Iterator for FoldedPages<'_> {
        type Item = Rc<Page>;

        fn next(&mut self) -> Option<Self::Item> {
            self.folds.next().map(|fold| {
                self.page = Rc::new(self.page.fold(fold));
                self.page.clone()
            })
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 13,
    name: "Transparent Origami",
    preprocessor: Some(|input| Ok(Box::new(Problem::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(Answer::Unsigned(
                input
                    .expect_data::<Problem>()?
                    .apply_folds()
                    .next()
                    .unwrap()
                    .num_dots()
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            // Process
            let last_page = input
                .expect_data::<Problem>()?
                .apply_folds()
                .last()
                .unwrap();

            // This is a little annoying because it requires looking at letters in the folded image,
            // which cannot really be done in automated way easily.
            println!("Part two folded image:\n");
            println!("{last_page:?}");
            println!("Part two actual answer: JGAJEFKU\n");

            Ok(Answer::Unsigned(last_page.num_dots().try_into().unwrap()))
        },
    ],
};
