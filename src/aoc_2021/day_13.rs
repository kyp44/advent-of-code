use std::{collections::HashSet, fmt::Debug, rc::Rc, str::FromStr};

use cgmath::Vector2;
use nom::{
    bytes::complete::tag,
    character::complete::{multispace1, one_of},
    combinator::map,
    sequence::{preceded, separated_pair},
};

use crate::aoc::{parse::trim, prelude::*};

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

#[derive(PartialEq, Eq, Hash, Clone)]
struct Point {
    point: Vector2<isize>,
}
impl Parseable<'_> for Point {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        map(
            separated_pair(
                nom::character::complete::i32,
                trim(false, tag(",")),
                nom::character::complete::i32,
            ),
            |(x, y)| Self {
                point: Vector2::new(x.try_into().unwrap(), y.try_into().unwrap()),
            },
        )(input)
    }
}
impl Point {
    fn new(x: isize, y: isize) -> Self {
        Self {
            point: Vector2::new(x, y),
        }
    }
}

#[derive(Clone)]
struct Page {
    dots: HashSet<Point>,
}
impl FromStr for Page {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Page {
            dots: Point::gather(s.lines())?.into_iter().collect(),
        })
    }
}
impl Debug for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let grid = Grid::from_coordinates(self.dots.iter().map(|p| p.point));
        writeln!(f, "{grid:?}")
    }
}
impl Page {
    fn fold(&self, fold: &Fold) -> Self {
        let mut dots = HashSet::new();
        match fold {
            Fold::Vertical(fx) => {
                for dot in self.dots.iter().map(|p| p.point) {
                    dots.insert(Point::new(
                        if dot.x <= *fx { dot.x } else { 2 * fx - dot.x },
                        dot.y,
                    ));
                }
            }
            Fold::Horizontal(fy) => {
                for dot in self.dots.iter().map(|p| p.point) {
                    dots.insert(Point::new(
                        dot.x,
                        if dot.y <= *fy { dot.y } else { 2 * fy - dot.y },
                    ));
                }
            }
        }
        Self { dots }
    }

    fn len(&self) -> usize {
        self.dots.len()
    }
}

#[derive(Clone, Copy)]
enum Fold {
    Vertical(isize),
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

struct Problem {
    page: Page,
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
    fn apply_folds(&self) -> FoldedPages {
        FoldedPages {
            page: Rc::new(self.page.clone()),
            folds: self.folds.iter(),
        }
    }
}
struct FoldedPages<'a> {
    page: Rc<Page>,
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

pub const SOLUTION: Solution = Solution {
    day: 13,
    name: "Transparent Origami",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let problem = Problem::from_str(input.expect_input()?)?;
            let first_fold = problem.apply_folds().next().unwrap();

            //println!("{:?}\n", problem.page);
            //println!("{:?}\n", first_fold);

            // Process
            Ok(Answer::Unsigned(first_fold.len().try_into().unwrap()))
        },
        // Part two
        |input| {
            // Generation
            let problem = Problem::from_str(input.expect_input()?)?;

            // This is a little annoying because it requires looking at letters in the folded image,
            // which cannot reallly be done in automated way easily.
            let last_page = problem.apply_folds().last().unwrap();
            println!("Part two folded image:\n");
            println!("{last_page:?}");
            println!("Part two actual answer: JGAJEFKU\n");

            // Process
            Ok(Answer::Unsigned(last_page.len().try_into().unwrap()))
        },
    ],
};
