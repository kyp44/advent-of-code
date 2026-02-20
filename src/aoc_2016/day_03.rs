use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "5 10 25";
            answers = unsigned![0];
        }
        example {
            input = "101 301 501
102 302 502
103 303 503
201 401 601
202 402 602
203 403 603";
            answers = unsigned![3, 6];
        }
        actual_answers = unsigned![869, 1544];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use nom::combinator::map;

    /// The lengths of three sides that may or may not be able to form a
    /// triangle.
    ///
    /// Can be parsed from text input, which are parsed from a single line.
    #[derive(Debug)]
    pub struct PossibleTriangle([u32; 3]);
    impl Parsable for PossibleTriangle {
        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(
                (
                    trim(false, nom::character::complete::u32),
                    trim(false, nom::character::complete::u32),
                    trim(false, nom::character::complete::u32),
                ),
                |(a, b, c)| Self([a, b, c]),
            )
            .parse(input)
        }
    }
    impl PossibleTriangle {
        /// Returns whether or not the lengths can form a triangle.
        pub fn is_triangle(&self) -> bool {
            let a = &self.0;
            a[0] + a[1] > a[2] && a[0] + a[2] > a[1] && a[1] + a[2] > a[0]
        }
    }

    /// Something that can count the number of triangles.
    pub trait TriangleCounter {
        /// Returns an [`Iterator`] over [`PossibleTriangle`]s to be counted.
        fn get_triangles(&self) -> impl Iterator<Item = &PossibleTriangle>;

        /// Counts the number of [`PossibleTriangle`]s returned by
        /// [`get_triangles`](TriangleCounter::get_triangles)
        /// that can actually be triangles.
        fn count_triangles(&self) -> u64 {
            self.get_triangles().filter_count(|pt| pt.is_triangle())
        }
    }

    /// Possible triangles that are parsed horizontally from text, with one
    /// triangle per line as in part one.
    pub struct HorizontalTriangles {
        /// The list of possible triangles.
        triangles: Vec<PossibleTriangle>,
    }
    impl FromStr for HorizontalTriangles {
        type Err = NomParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self {
                triangles: PossibleTriangle::gather(s.lines())?,
            })
        }
    }
    impl TriangleCounter for HorizontalTriangles {
        fn get_triangles(&self) -> impl Iterator<Item = &PossibleTriangle> {
            self.triangles.iter()
        }
    }

    /// Possible triangles that are parsed vertically from text, with one
    /// triangle taking a part of three lines as in part two.
    pub struct VerticalTriangles {
        /// The list of possible triangles.
        triangles: Vec<PossibleTriangle>,
    }
    impl FromStr for VerticalTriangles {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // First get horizontal triangles
            let hor_triangles = PossibleTriangle::gather(s.lines())?;

            // Verify that the number of rows result in complete triangles
            (hor_triangles.len().is_multiple_of(3)).ok_or(AocError::InvalidInput(
                "The number of rows do not make for complete vertical triangles ".into(),
            ))?;

            // Now rearrange them
            let mut triangles = Vec::new();
            for chunk in hor_triangles.chunks(3) {
                for i in 0..3 {
                    triangles.push(PossibleTriangle([
                        chunk[0].0[i],
                        chunk[1].0[i],
                        chunk[2].0[i],
                    ]))
                }
            }

            Ok(Self { triangles })
        }
    }
    impl TriangleCounter for VerticalTriangles {
        fn get_triangles(&self) -> impl Iterator<Item = &PossibleTriangle> {
            self.triangles.iter()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 3,
    name: "Squares With Three Sides",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Generation
            let triangles = HorizontalTriangles::from_str(input.expect_text()?)?;

            // Process
            Ok(triangles.count_triangles().into())
        },
        // Part two
        |input| {
            // Generation
            let triangles = VerticalTriangles::from_str(input.expect_text()?)?;

            // Process
            Ok(triangles.count_triangles().into())
        },
    ],
};
