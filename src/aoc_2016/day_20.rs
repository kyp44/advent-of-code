use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "5-8
0-2
4-7";
            answers = unsigned![3, 4294967288];
        }
        example {
            input = "1-10
11-20
23-30
31-4294967290";
            answers = unsigned![0, 8];
        }
        actual_answers = unsigned![31053880, 117];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::inclusive_range;
    use core::ops::RangeInclusive;
    use itertools::Itertools;
    use nom::combinator::map;

    /// An inclusive IP range, which has an order defined based on the start of
    /// the range.
    ///
    /// Can be parsed from text input.
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct IpRange(pub RangeInclusive<u32>);
    impl Parsable for IpRange {
        type Parsed<'a> = Self;

        fn parser<'a>(input: &'a str) -> NomParseResult<&'a str, Self::Parsed<'a>> {
            map(inclusive_range(nom::character::complete::u32), Self).parse(input)
        }
    }
    impl IpRange {
        /// Returns whether the ranges overlap.
        pub fn overlaps(&self, other: &Self) -> bool {
            self.0.contains(other.0.start()) || other.0.contains(self.0.start())
        }

        /// Returns whether the ranges are adjacent since the underlying type is
        /// an integer.
        pub fn is_adjacent(&self, other: &Self) -> bool {
            (*self.0.end() < u32::MAX && self.0.end() + 1 == *other.0.start())
                || (*other.0.end() < u32::MAX && *other.0.end() + 1 == *self.0.start())
        }

        /// Returns the the union of the ranges if they overlap or are adjacent.
        pub fn union(&self, other: &Self) -> Option<Self> {
            (self.overlaps(other) || self.is_adjacent(other)).then(|| {
                Self(RangeInclusive::new(
                    (*self.0.start()).min(*other.0.start()),
                    (*self.0.end()).max(*other.0.end()),
                ))
            })
        }
    }
    impl PartialOrd for IpRange {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for IpRange {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.0.start().cmp(other.0.start())
        }
    }

    /// A set of blocked IP ranges in order and a guaranteed to not be
    /// overlapping or adjacent.
    ///
    /// Can be parsed from text input.
    pub struct IpBlacklist {
        /// The blocked IP ranges, in order.
        ranges: Vec<IpRange>,
    }
    impl FromStr for IpBlacklist {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(IpRange::gather(s.lines()).map(|mut orig_ranges| {
                orig_ranges.sort();

                let mut ranges: Vec<IpRange> = Vec::new();

                for range in orig_ranges.into_iter() {
                    match ranges.last_mut() {
                        Some(curr_range) => match curr_range.union(&range) {
                            Some(combined_range) => {
                                *curr_range = combined_range;
                            }
                            None => {
                                ranges.push(range);
                            }
                        },
                        None => {
                            ranges.push(range);
                        }
                    }
                }

                Self { ranges }
            })?)
        }
    }
    impl IpBlacklist {
        /// Returns the lowest unblocked IP, if there is one, that is, if not
        /// every IP is blocked.
        pub fn lowest_ip(&self) -> Option<u32> {
            // This all depends on the fact that the ranges are sorted and not adjacent.
            let first_ran = match self.ranges.first() {
                Some(r) => r,
                None => return Some(0),
            };
            let first_end = *first_ran.0.end();

            if 0 < *first_ran.0.start() {
                Some(0)
            } else if first_end < u32::MAX {
                Some(first_end + 1)
            } else {
                None
            }
        }

        /// Returns the number of unblocked IPs.
        pub fn num_unblocked_ips(&self) -> u32 {
            // NOTE: This all depends on the fact that the ranges are sorted and not
            // overlapping

            // See if any are below all the ranges
            let first_ran = match self.ranges.first() {
                Some(r) => r,
                None => return u32::MAX,
            };
            let mut num_unblocked = *first_ran.0.start();

            for (ra, rb) in self.ranges.iter().tuple_windows() {
                num_unblocked += *rb.0.start() - *ra.0.end() - 1;
            }

            // Check for any above the last range
            if let Some(last_ran) = self.ranges.last()
                && *last_ran.0.end() < u32::MAX
            {
                num_unblocked += u32::MAX - *last_ran.0.end();
            }

            num_unblocked
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 20,
    name: "Firewall Rules",
    preprocessor: Some(|input| Ok(Box::new(IpBlacklist::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(u64::from(
                input
                    .expect_data::<IpBlacklist>()?
                    .lowest_ip()
                    .ok_or(AocError::NoSolution)?,
            )
            .into())
        },
        // Part two
        |input| {
            // Process
            Ok(u64::from(input.expect_data::<IpBlacklist>()?.num_unblocked_ips()).into())
        },
    ],
};
