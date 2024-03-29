use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "--- scanner 0 ---
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
30,-46,-14";
            answers = unsigned![79, 3621];
        }
        actual_answers = unsigned![438, 11985];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use derive_more::{Deref, From};
    use derive_new::new;
    use euclid::default::{Point3D, Rotation3D, Vector3D};
    use itertools::{iproduct, Itertools};
    use maplit::hashset;
    use nom::{
        bytes::complete::tag,
        combinator::map,
        multi::separated_list1,
        sequence::{delimited, preceded},
        Finish,
    };
    use std::{
        collections::{HashMap, HashSet},
        ops::Sub,
        rc::Rc,
    };
    use std::{hash::Hash, ops::Add};
    use strum::IntoEnumIterator;
    use strum_macros::EnumIter;

    /// A 3D vector over the field of integers.
    type Vector = Vector3D<i32>;

    /// A 3D point in our coordinate system, which can be parsed from text input.
    #[derive(Deref, Debug, Clone, Copy, PartialEq, Eq, Hash, From)]
    pub struct Point(Point3D<i32>);
    impl Parsable<'_> for Point {
        fn parser(input: &str) -> NomParseResult<&str, Self> {
            map(
                separated_list1(tag(","), trim(false, nom::character::complete::i32)),
                |v| Self(Point3D::new(v[0], v[1], v[2])),
            )(input)
        }
    }
    impl From<Point> for Rotation3D<i32> {
        fn from(p: Point) -> Self {
            Self::from_sv(0, p.to_vector())
        }
    }
    impl Add<Vector> for Point {
        type Output = Self;

        fn add(self, rhs: Vector) -> Self::Output {
            (*self + rhs).into()
        }
    }
    impl Sub for Point {
        type Output = Vector;

        fn sub(self, rhs: Self) -> Self::Output {
            *self - *rhs
        }
    }
    impl Sub<Vector> for Point {
        type Output = Self;

        fn sub(self, rhs: Vector) -> Self::Output {
            (*self - rhs).into()
        }
    }
    impl From<Rotation3D<i32>> for Point {
        fn from(q: Rotation3D<i32>) -> Self {
            q.vector_part().to_point().into()
        }
    }

    /// Extension trait for [`Rotation3D`] that allow raw mathematical operations
    /// on the quaternions.
    ///
    /// Note that these could not have been implemented as the normal operator traits
    /// due to the orphan rule.
    trait QuaternionExt {
        /// Creates a quaternion from the scalar and vector components.
        fn from_sv(s: i32, v: Vector3D<i32>) -> Self;
        /// Conjugates a quaternion.
        fn conj(self) -> Self;
        /// Multiplies two quaternions.
        fn mul(self, rhs: Self) -> Self;
        /// Divides two quaternions.
        fn div(self, rhs: i32) -> Self;
    }
    impl QuaternionExt for Rotation3D<i32> {
        fn from_sv(s: i32, v: Vector3D<i32>) -> Self {
            Self::quaternion(v.x, v.y, v.z, s)
        }

        fn conj(self) -> Self {
            Self::from_sv(self.r, -self.vector_part())
        }

        fn mul(self, rhs: Self) -> Self {
            Self::quaternion(
                self.r * rhs.i + self.i * rhs.r + self.j * rhs.k - self.k * rhs.j,
                self.r * rhs.j + self.j * rhs.r + self.k * rhs.i - self.i * rhs.k,
                self.r * rhs.k + self.k * rhs.r + self.i * rhs.j - self.j * rhs.i,
                self.r * rhs.r - self.i * rhs.i - self.j * rhs.j - self.k * rhs.k,
            )
        }

        fn div(self, rhs: i32) -> Self {
            Self::from_sv(self.r / rhs, self.vector_part() / rhs)
        }
    }

    /// 2D orthogonal rotation angles.
    #[derive(EnumIter)]
    enum RotationAngle {
        /// 0 degrees.
        Rot0,
        /// 90 degrees counter-clockwise.
        Rot90,
        /// 180 degrees.
        Rot180,
        /// 270 degrees counter-clockwise.
        Rot270,
    }
    impl RotationAngle {
        /// Generates a rotation quaternion from the rotation angle about a particular
        /// axis, which must be a unit vector.
        fn rotation_quaternion(&self, unit_axis: Vector) -> RotationQuaternion {
            match self {
                RotationAngle::Rot0 => {
                    RotationQuaternion::new(1, Rotation3D::from_sv(1, Vector::zero()))
                }
                RotationAngle::Rot90 => {
                    RotationQuaternion::new(2, Rotation3D::from_sv(1, unit_axis))
                }
                RotationAngle::Rot180 => {
                    RotationQuaternion::new(1, Rotation3D::from_sv(0, unit_axis))
                }
                RotationAngle::Rot270 => {
                    RotationQuaternion::new(2, Rotation3D::from_sv(-1, unit_axis))
                }
            }
        }
    }

    /// A quaternion that performs a rotation about the origin.
    #[derive(new, Clone, Debug)]
    struct RotationQuaternion {
        /// Divisor needed to account for the sine and cosine when using integers.
        ///
        /// This is the square of the divisor of the actual rotation quaternion so
        /// that when rotation is applied we need only divide by this at the end
        /// once.
        divisor: i32,
        /// The rotation quaternion without the divisor.
        quat: Rotation3D<i32>,
    }
    impl RotationQuaternion {
        /// Returns the identity rotation quaternion that leaves points unchanged.
        fn identity() -> Self {
            Self::new(1, Rotation3D::identity())
        }

        /// Rotates a point according to this quaternion.
        fn rotate_point(&self, point: Point) -> Point {
            self.quat
                .mul(point.into())
                .mul(self.quat.conj())
                .div(self.divisor)
                .into()
        }

        /// Generates a new rotation quaternion that is this one followed by another.
        fn compose(self, other: Self) -> Self {
            Self {
                divisor: self.divisor * other.divisor,
                quat: other.quat.mul(self.quat),
            }
        }

        /// Iterates over the 24 possible rotation quaternions representing possible scanner
        /// orientations.
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
                .map(|(fr, ra)| ra.rotation_quaternion(Vector::unit_x()).compose(fr))
        }
    }

    /// Relation of one Scanner to another.
    #[derive(Clone, Debug)]
    struct Transposer {
        /// The location of scanner B in the coordinate system of scanner A.
        location: Point,
        /// The rotation needed to bring points relative to scanner B into the
        /// coordinate system of scanner A prior to translating.
        rotation: RotationQuaternion,
    }
    impl Transposer {
        /// Returns the transposer that leaves points unchanged.
        fn identity() -> Self {
            Transposer {
                location: Point(Point3D::origin()),
                rotation: RotationQuaternion::identity(),
            }
        }

        /// Transposes a point relative to scanner B to be relative
        /// to scanner A.
        fn transpose_point(&self, point: Point) -> Point {
            self.rotation.rotate_point(point) + self.location.to_vector()
        }

        /// Composes transpositions.
        ///
        /// If this is a transposer from scanner B to A, and `other` is from C
        /// to B, then the result transposes C to A.
        fn compose(self, other: Self) -> Self {
            Self {
                location: self.rotation.rotate_point(other.location) + self.location.to_vector(),
                rotation: other.rotation.compose(self.rotation),
            }
        }
    }

    /// A scanner and the beacons detected by it, which can be parsed from
    /// text input.
    #[derive(Debug, Eq)]
    struct Scanner {
        /// The scanner number.
        number: u8,
        /// The beacon locations relative to this scanner.
        beacon_points: Box<[Point]>,
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

            Ok(Self {
                number,
                beacon_points: points,
            })
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
        /// Tries to correlate another scanner with this one.
        ///
        /// Returns the transposer from this scanner to the other if
        /// correlation was successful.
        fn try_to_correlate(&self, other: &Self) -> Option<Transposer> {
            // First try every possible orientation
            for rotation in RotationQuaternion::orientations() {
                // Try every pairing of points to find the relative difference
                let other_points: HashSet<Point> = other
                    .beacon_points
                    .iter()
                    .map(|p| rotation.rotate_point(*p))
                    .collect();
                for (ps, po) in iproduct!(self.beacon_points.iter(), other_points.iter()) {
                    let delta = *ps - *po;
                    if self
                        .beacon_points
                        .iter()
                        .filter(|p| other_points.contains(&(**p - delta)))
                        .count()
                        >= 12
                    {
                        // We have a sufficient number of correlated points!
                        return Some(Transposer {
                            location: delta.to_point().into(),
                            rotation,
                        });
                    }
                }
            }
            None
        }
    }

    /// Map of scanners to the transpositions necessary to bring points in the
    /// coordinate system of that scanner into that of scanner 0.
    type CorrelationMap = HashMap<Rc<Scanner>, Transposer>;

    /// The network of scanners, which can be parsed from text input.
    pub struct ScannerNetwork {
        /// The list of scanners.
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
        /// Correlates all the scanners together and return the correlated network.
        pub fn correlate(&self) -> CorrelatedScannerNetwork {
            /// This is an internal function of [`ScannerNetwork::correlate`] that recursively correlates
            /// scanners one by one.
            fn correlate_rec(
                from: Rc<Scanner>,
                scanners: &[Rc<Scanner>],
                correlated: &mut HashSet<Rc<Scanner>>,
            ) -> CorrelationMap {
                // Try every scanner that is not already correlated
                let mut correlations = CorrelationMap::new();
                for to in scanners.iter() {
                    // If we've already correlated this one then move on.
                    if correlated.contains(to) {
                        continue;
                    }

                    if let Some(transposer) = from.try_to_correlate(to) {
                        // Add this to the list of correlated scanners
                        correlated.insert(to.clone());

                        // Now recurse to get with which uncorrelated scanners this is also correlated
                        // and map these additional sub-correlations back to the original scanner.
                        correlations.extend(
                            correlate_rec(to.clone(), scanners, correlated)
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
            let mut correlations = correlate_rec(
                self.scanners[0].clone(),
                &self.scanners,
                &mut hashset![self.scanners[0].clone()],
            );

            // Add an identity correlation
            correlations.insert(self.scanners[0].clone(), Transposer::identity());

            CorrelatedScannerNetwork { correlations }
        }
    }

    /// Fully correlated scanner network.
    pub struct CorrelatedScannerNetwork {
        /// The correlation map.
        correlations: CorrelationMap,
    }
    impl CorrelatedScannerNetwork {
        /// Returns an [`Iterator`] of the coordinates of every beacon relative
        /// to scanner 0.
        pub fn beacons(&self) -> impl Iterator<Item = Point> + '_ {
            self.correlations
                .iter()
                .flat_map(|(scanner, transposer)| {
                    scanner
                        .beacon_points
                        .iter()
                        .map(|p| transposer.transpose_point(*p))
                })
                .unique()
        }

        /// Determines the maximum Manhattan distance between any two scanners.
        pub fn max_scanner_distance(&self) -> i32 {
            iproduct!(self.correlations.values(), self.correlations.values())
                .map(|(ta, tb)| (ta.location - tb.location).manhattan_len())
                .max()
                .unwrap()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 19,
    name: "Beacon Scanner",
    preprocessor: Some(|input| Ok(Box::new(ScannerNetwork::from_str(input)?.correlate()).into())),
    solvers: &[
        // Part one
        |input| {
            Ok(Answer::Unsigned(
                input
                    .expect_data::<CorrelatedScannerNetwork>()?
                    .beacons()
                    .count()
                    .try_into()
                    .unwrap(),
            ))
        },
        // Part two
        |input| {
            // Processing
            Ok(Answer::Unsigned(
                input
                    .expect_data::<CorrelatedScannerNetwork>()?
                    .max_scanner_distance()
                    .try_into()
                    .unwrap(),
            ))
        },
    ],
};
