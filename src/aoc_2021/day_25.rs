use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Unsigned;

    solution_tests! {
    example {
        input = "v...>>.vv>
.vv>>.vv..
>>.>v>...v
>>v>>.>.v.
v>v.vv.v..
>.>>..v...
.vv..>.>v.
v.v..>>v.v
....v..v.>";
        answers = vec![123u64].answer_vec();
    }
    actual_answers = vec![Unsigned(123)];
    }
}

/// Contains solution implementation items.
mod solution {
    use cgmath::Vector2;

    use super::*;
    use std::fmt;

    #[derive(PartialEq, Eq, Clone, Default)]
    pub enum Location {
        #[default]
        Empty,
        East,
        South,
    }
    impl From<&Location> for char {
        fn from(value: &Location) -> Self {
            match value {
                Location::Empty => '.',
                Location::East => '>',
                Location::South => 'v',
            }
        }
    }
    impl fmt::Debug for Location {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", char::from(self))
        }
    }
    impl TryFrom<char> for Location {
        type Error = ();

        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                '.' => Ok(Self::Empty),
                '>' => Ok(Self::East),
                'v' => Ok(Self::South),
                _ => Err(()),
            }
        }
    }

    #[derive(Clone)]
    pub struct Trench {
        grid: Grid<Location>,
    }
    impl From<Grid<Location>> for Trench {
        fn from(value: Grid<Location>) -> Self {
            Trench { grid: value }
        }
    }
    impl fmt::Debug for Trench {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "{:?}", self.grid)
        }
    }
    impl Trench {
        fn specific_points<'a>(
            &'a self,
            location: &'a Location,
        ) -> impl Iterator<Item = AnyGridPoint> + 'a {
            self.grid.all_points().filter_map(move |point| {
                if self.grid.get(&point) == location {
                    Some(point.try_point_into().unwrap())
                } else {
                    None
                }
            })
        }

        fn next(&self) -> Self {
            let mut new_trench = Self::default(*self.grid.size());

            // First move East-moving cucumbers
            for point in self.specific_points(&Location::East) {
                let new_point = if self
                    .grid
                    .get(&(point + Vector2::new(1, 0)).unwrap_point(self.grid.size()))
                    == &Location::East
                {
                    point.try_point_into().unwrap()
                } else {
                    point.try_point_into().unwrap()
                };
                new_trench.grid.set(&new_point, Location::East)
            }

            new_trench
        }
    }
}

use solution::*;

pub const SOLUTION: Solution = Solution {
    day: 25,
    name: "Sea Cucumber",
    preprocessor: Some(|input| Ok(Box::new(Trench::from_grid_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Generation
            let trench = input.expect_data::<Trench>()?;

            println!("TODO:\n{:?}", trench);

            // Process
            Ok(0u64.into())
        },
    ],
};
