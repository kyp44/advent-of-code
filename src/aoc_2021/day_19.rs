use aoc::{parse::trim, prelude::*};
use cgmath::{Quaternion, Vector3, Zero};
use derive_new::new;
use itertools::iproduct;
use maplit::hashset;
use nom::{
    bytes::complete::tag,
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, preceded},
    Finish,
};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;
use std::rc::Rc;
use std::{collections::HashSet, str::FromStr};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(438), Unsigned(11985)],
    "--- scanner 0 ---
404,-588,-901
528,-643,409
-838,591,734
390,-675,-793
-537,-823,-458
-485,-357,347
-345,-311,381
-661,-816,-575
-876,649,763
-618,-824,-621
553,345,-567
474,580,667
-447,-329,318
-584,868,-557
544,-627,-890
564,392,-477
455,729,728
-892,524,684
-689,845,-530
423,-701,434
7,-33,-71
630,319,-379
443,580,662
-789,900,-551
459,-707,401

--- scanner 1 ---
686,422,578
605,423,415
515,917,-361
-336,658,858
95,138,22
-476,619,847
-340,-569,-846
567,-361,727
-460,603,-452
669,-402,600
729,430,532
-500,-761,534
-322,571,750
-466,-666,-811
-429,-592,574
-355,545,-477
703,-491,-529
-328,-685,520
413,935,-424
-391,539,-444
586,-435,557
-364,-763,-893
807,-499,-711
755,-354,-619
553,889,-390

--- scanner 2 ---
649,640,665
682,-795,504
-784,533,-524
-644,584,-595
-588,-843,648
-30,6,44
-674,560,763
500,723,-460
609,671,-379
-555,-800,653
-675,-892,-343
697,-426,-610
578,704,681
493,664,-388
-671,-858,530
-667,343,800
571,-461,-707
-138,-166,112
-889,563,-600
646,-828,498
640,759,510
-630,509,768
-681,-892,-333
673,-379,-804
-742,-814,-386
577,-820,562

--- scanner 3 ---
-589,542,597
605,-692,669
-500,565,-823
-660,373,557
-458,-679,-417
-488,449,543
-626,468,-788
338,-750,-386
528,-832,-391
562,-778,733
-938,-730,414
543,643,-506
-524,371,-870
407,773,750
-104,29,83
378,-903,-323
-778,-728,485
426,699,580
-438,-605,-362
-469,-447,-387
509,732,623
647,635,-688
-868,-804,481
614,-800,639
595,780,-596

--- scanner 4 ---
727,592,562
-293,-554,779
441,611,-461
-714,465,-776
-743,427,-804
-660,-479,-426
832,-632,460
927,-485,-438
408,393,-506
466,436,-512
110,16,151
-258,-428,682
-393,719,612
-211,-452,876
808,-476,-593
-575,615,604
-485,667,467
-680,325,-822
-627,-443,-432
872,-547,-609
833,512,582
807,604,487
839,-516,451
891,-625,532
-652,-548,-490
30,-46,-14",
        vec![79u64, 3621].answer_vec()
    }
}

type Vector = Vector3<i32>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    vect: Vector,
}
impl From<Vector> for Point {
    fn from(vect: Vector) -> Self {
        Point { vect }
    }
}
impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        (self.vect + rhs.vect).into()
    }
}
impl Parseable<'_> for Point {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        map(
            separated_list1(tag(","), trim(false, nom::character::complete::i32)),
            |vec| Self {
                vect: Vector::new(vec[0], vec[1], vec[2]),
            },
        )(input)
    }
}
impl From<Point> for Quaternion<i32> {
    fn from(p: Point) -> Self {
        Self::from_sv(0, p.vect)
    }
}
impl From<Quaternion<i32>> for Point {
    fn from(q: Quaternion<i32>) -> Self {
        Point { vect: q.v }
    }
}
impl Point {
    fn manhatten_distance(&self, other: &Self) -> i32 {
        let d = self.vect - other.vect;
        d.x.abs() + d.y.abs() + d.z.abs()
    }
}

/// Ugh, these aren't implemented for integer base types for some reason
trait QuaternionExt {
    fn conj(self) -> Self;
    fn mul(self, rhs: Self) -> Self;
    fn div(self, rhs: i32) -> Self;
}
impl QuaternionExt for Quaternion<i32> {
    fn conj(self) -> Self {
        Quaternion::from_sv(self.s, -self.v)
    }

    fn mul(self, rhs: Self) -> Self {
        Self::new(
            self.s * rhs.s - self.v.x * rhs.v.x - self.v.y * rhs.v.y - self.v.z * rhs.v.z,
            self.s * rhs.v.x + self.v.x * rhs.s + self.v.y * rhs.v.z - self.v.z * rhs.v.y,
            self.s * rhs.v.y + self.v.y * rhs.s + self.v.z * rhs.v.x - self.v.x * rhs.v.z,
            self.s * rhs.v.z + self.v.z * rhs.s + self.v.x * rhs.v.y - self.v.y * rhs.v.x,
        )
    }

    fn div(self, rhs: i32) -> Self {
        Self::from_sv(self.s / rhs, self.v / rhs)
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
    fn rotation_quaternion(&self, unit_axis: Vector) -> RotationQuaternion {
        match self {
            RotationAngle::Rot0 => {
                RotationQuaternion::new(1, Quaternion::from_sv(1, Vector::zero()))
            }
            RotationAngle::Rot90 => RotationQuaternion::new(2, Quaternion::from_sv(1, unit_axis)),
            RotationAngle::Rot180 => RotationQuaternion::new(1, Quaternion::from_sv(0, unit_axis)),
            RotationAngle::Rot270 => RotationQuaternion::new(2, Quaternion::from_sv(-1, unit_axis)),
        }
    }
}

#[derive(new, Clone, Debug)]
struct RotationQuaternion {
    divisor: i32,
    quat: Quaternion<i32>,
}
impl RotationQuaternion {
    /// Identity rotation that leaves points unchanged.
    fn identity() -> Self {
        Self::new(1, Quaternion::from_sv(1, Vector::zero()))
    }

    /// Rotates a point.
    fn rotate_point(&self, point: Point) -> Point {
        self.quat
            .mul(point.into())
            .mul(self.quat.conj())
            .div(self.divisor)
            .into()
    }

    // Generates a new rotation that is this one followed by another.
    fn and_then(self, other: Self) -> Self {
        Self {
            divisor: self.divisor * other.divisor,
            quat: other.quat.mul(self.quat),
        }
    }

    /// Iterates over the 24 possible rotation function representing possible scanner orientations
    fn orientations() -> impl Iterator<Item = Self> {
        let facing_rotations: [RotationQuaternion; 6] = [
            RotationAngle::Rot0.rotation_quaternion(Vector::unit_z()),
            RotationAngle::Rot90.rotation_quaternion(Vector::unit_z()),
            RotationAngle::Rot180.rotation_quaternion(Vector::unit_z()),
            RotationAngle::Rot270.rotation_quaternion(Vector::unit_z()),
            RotationAngle::Rot90.rotation_quaternion(Vector::unit_y()),
            RotationAngle::Rot270.rotation_quaternion(Vector::unit_y()),
        ];

        iproduct!(facing_rotations.into_iter(), RotationAngle::iter())
            .map(|(fr, ra)| ra.rotation_quaternion(Vector::unit_x()).and_then(fr))
    }
}

/// Relation of one Scanner to another.
/// The Transposer of scanner A to scanner B represents the location of
/// scanner B in the coordinate system of scanner A, and the rotation
/// needed to bring points relative to scanner A into the coordinate
/// system of scanner A prior to translating.
#[derive(Clone, Debug)]
struct Transposer {
    location: Point,
    rotation: RotationQuaternion,
}
impl Transposer {
    /// Identity tranposer that leaves points unchanged.
    fn identity() -> Self {
        Transposer {
            location: Vector::zero().into(),
            rotation: RotationQuaternion::identity(),
        }
    }

    /// Transposes a point relative to scanner B to be relative
    /// to scanner A.
    fn transpose_point(&self, point: Point) -> Point {
        self.rotation.rotate_point(point) + self.location
    }

    /// Composes transpositions.
    /// If this is a transposer from scanner B to A, and other is from C
    /// to B, then the result transposes C to A.
    fn compose(self, other: Self) -> Self {
        Self {
            location: self.rotation.rotate_point(other.location) + self.location,
            rotation: other.rotation.and_then(self.rotation),
        }
    }
}

#[derive(Debug, Eq)]
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
            trim(
                false,
                preceded(tag("scanner "), nom::character::complete::u8),
            ),
            tag(sep),
        )(s)
        .finish()?;

        let points = Point::gather(s.trim().lines())?.into_boxed_slice();

        Ok(Self { number, points })
    }
}
impl PartialEq for Scanner {
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}
impl Hash for Scanner {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.number.hash(state);
    }
}
impl Scanner {
    /// Try to correlate a scanner with this one.
    /// Returns the Transposer from this scanner to the other.
    fn try_to_correlate(&self, other: &Self) -> Option<Transposer> {
        // First try every possible orientation
        for rotation in RotationQuaternion::orientations() {
            // Try every pairing of points to find the relative difference
            let other_points: HashSet<Vector> = other
                .points
                .iter()
                .map(|p| rotation.rotate_point(*p).vect)
                .collect();
            for (ps, po) in iproduct!(self.points.iter(), other_points.iter()) {
                let delta = ps.vect - po;
                if self
                    .points
                    .iter()
                    .filter(|p| other_points.contains(&(p.vect - delta)))
                    .count()
                    >= 12
                {
                    // We have a sufficient number of correlated points!
                    return Some(Transposer {
                        location: delta.into(),
                        rotation,
                    });
                }
            }
        }
        None
    }
}

type Correlations = HashMap<Rc<Scanner>, Transposer>;

struct ScannerNetwork {
    scanners: Box<[Rc<Scanner>]>,
}
impl FromStr for ScannerNetwork {
    type Err = AocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            scanners: s
                .split("\n\n")
                .map(|ss| Ok(Rc::new(Scanner::from_str(ss)?)))
                .collect::<AocResult<Box<[Rc<Scanner>]>>>()?,
        })
    }
}
impl ScannerNetwork {
    fn correlate(&self) -> Correlations {
        // Recursively correlate all scanners
        fn correlate_rec(
            from: Rc<Scanner>,
            scanners: &[Rc<Scanner>],
            correlated: HashSet<Rc<Scanner>>,
        ) -> Correlations {
            // Try every scanner that is not already correlated
            let mut correlations = Correlations::new();
            for to in scanners.iter().filter(|s| !correlated.contains(*s)) {
                if let Some(transposer) = from.try_to_correlate(to) {
                    // Add this to the list of correlated scanners
                    let mut new_correlated = correlated.clone();
                    new_correlated.insert(to.clone());

                    // Now recurse to get with which uncorrelated scanners this is also correlated
                    // and map these additional sub-correlations back to the original scanner.
                    correlations.extend(
                        correlate_rec(to.clone(), scanners, new_correlated)
                            .into_iter()
                            .map(|(s, t)| (s, transposer.clone().compose(t))),
                    );

                    // Add this correlation
                    correlations.insert(to.clone(), transposer);
                }
            }
            correlations
        }

        // Get all scanners relative to scanner 0
        correlate_rec(
            self.scanners[0].clone(),
            &self.scanners,
            hashset![self.scanners[0].clone()],
        )
    }
}

// Expensive data to preprocess for both parts
pub struct BeaconScannerData {
    correlations: Correlations,
}

pub const SOLUTION: Solution = Solution {
    day: 19,
    name: "Beacon Scanner",
    preprocessor: Some(|input| {
        // Generation
        let network = ScannerNetwork::from_str(input)?;

        // Correlate
        let mut correlations = network.correlate();
        // Add an identity correlation
        correlations.insert(network.scanners[0].clone(), Transposer::identity());

        Ok(Box::new(BeaconScannerData { correlations }).into())
    }),
    solvers: &[
        // Part one
        |input| {
            // Now build a set of all points (beacons) relative to scanner 0
            let data = input.expect_data::<BeaconScannerData>()?;

            let mut points: HashSet<Point> = HashSet::new();
            for (scanner, transposer) in data.correlations.iter() {
                points.extend(
                    scanner
                        .points
                        .iter()
                        .map(|p| transposer.transpose_point(*p)),
                );
            }

            Ok(Answer::Unsigned(points.len().try_into().unwrap()))
        },
        // Part two
        |input| {
            // Generation
            let data = input.expect_data::<BeaconScannerData>()?;

            // Processing
            Ok(Answer::Unsigned(
                iproduct!(data.correlations.values(), data.correlations.values())
                    .map(|(ta, tb)| ta.location.manhatten_distance(&tb.location))
                    .max()
                    .unwrap()
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
