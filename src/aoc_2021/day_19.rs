use std::str::FromStr;

use cgmath::{Quaternion, Vector3};
use itertools::iproduct;
use nom::{
    bytes::complete::tag,
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, preceded},
    Finish,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::aoc::{parse::trim, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(123)],
    "--- scanner 0 ---
-1,-1,1
-2,-2,2
-3,-3,3
-2,-3,1
5,6,-4
8,0,7",
    vec![123u64].answer_vec()
    }
}

type Vector = Vector3<i32>;

#[derive(Debug, Clone, Copy)]
struct Point {
    vect: Vector,
}
impl Parseable<'_> for Point {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        map(
            separated_list1(tag(","), trim(nom::character::complete::i32)),
            |vec| Self {
                vect: Vector::new(vec[0], vec[1], vec[2]),
            },
        )(input)
    }
}

#[derive(Debug)]
struct Scanner {
    number: u8,
    points: Box<[Point]>,
}
impl FromStr for Scanner {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sep = "---";
        let (s, number) = delimited::<_, _, _, _, NomParseError, _, _, _>(
            tag(sep),
            trim(preceded(tag("scanner "), nom::character::complete::u8)),
            tag(sep),
        )(s)
        .finish()?;

        let points = Point::gather(s.trim().lines())?.into_boxed_slice();

        Ok(Self { number, points })
    }
}

/// Ugh, this isn't implemented for integer base types for some reason
trait ConjugateExt {
    fn conj(self) -> Self;
}
impl ConjugateExt for Quaternion<i32> {
    fn conj(self) -> Self {
        Quaternion::from_sv(self.s, -self.v)
    }
}

#[derive(EnumIter)]
enum RotationAngle {
    Rot0,
    Rot90,
    Rot180,
    Rot270,
}
impl RotationAngle {
    fn rotation_function(&self, unit_axis: Vector) -> impl Fn(&Point) -> Point {
        match self {
            RotationAngle::Rot0 => |p: &Point| *p,
            RotationAngle::Rot90 => |p: &Point| *p,
            RotationAngle::Rot180 => |p: &Point| {
                let unit = Quaternion::from_sv(0, unit_axis);
                Point {
                    vect: (unit * Quaternion::from_sv(0, p.vect) * unit.conj()).v,
                }
            },
            RotationAngle::Rot270 => |p: &Point| *p,
        }
    }
}

/// Iterates over the 24 possible rotation function representing possible scanner orientations
fn rotation_functions() -> impl Iterator<Item = impl Fn(Point) -> Point> {
    let orientation_vectors: Vec<Vector> = vec![
        Vector::unit_x(),
        -Vector::unit_x(),
        Vector::unit_y(),
        -Vector::unit_y(),
        Vector::unit_z(),
        -Vector::unit_z(),
    ];

    //iproduct!(orientation_vectors.iter(), RotationAngle::iter()).map(|(ov, ra)| |p| p)
    (0..5).map(|i| move |p: Point| Point { vect: p.vect * i })
}

pub const SOLUTION: Solution = Solution {
    day: 19,
    name: "Beacon Scanner",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let scanners = input
                .split("\n\n")
                .map(|ss| Scanner::from_str(ss))
                .collect::<AocResult<Box<[Scanner]>>>()?;

            println!("TODO: {:?}", scanners);

            // Process
            Ok(0u64.into())
        },
    ],
};
