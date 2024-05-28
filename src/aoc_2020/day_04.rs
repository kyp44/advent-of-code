use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;
    use Answer::Unsigned;

    solution_tests! {
        example {
            input = "ecl:gry pid:860033327 eyr:2020 hcl:#fffffd
byr:1937 iyr:2017 cid:147 hgt:183cm

iyr:2013 ecl:amb cid:350 eyr:2023 pid:028048884
hcl:#cfa07d byr:1929

hcl:#ae17e1 iyr:2013
eyr:2024
ecl:brn pid:760753108 byr:1931
hgt:179cm

hcl:#cfa07d eyr:2025 pid:166559648
iyr:2011 ecl:brn hgt:59in";
            answers = &[Some(Unsigned(2)), None];
        }
        example {
            input = "eyr:1972 cid:100
hcl:#18171d ecl:amb hgt:170 pid:186cm iyr:2018 byr:1926

iyr:2019
hcl:#602927 eyr:1967 hgt:170cm
ecl:grn pid:012533040 byr:1946

hcl:dab227 iyr:2012
ecl:brn hgt:182cm pid:021572410 eyr:2020 byr:1992 cid:277

hgt:59cm ecl:zzz
eyr:2038 hcl:74454a iyr:2023
pid:3556412378 byr:2007";
            answers = &[None, Some(Unsigned(0))];
        }
        example {
            input = "pid:087499704 hgt:74in ecl:grn iyr:2012 eyr:2030 byr:1980
hcl:#623a2f

eyr:2029 ecl:blu cid:129 byr:1989
iyr:2014 pid:896056539 hcl:#a97842 hgt:165cm

hcl:#888785
hgt:164cm byr:2001 iyr:2015 cid:88
pid:545766238 ecl:hzl
eyr:2022

iyr:2010 hgt:158cm hcl:#b6652a ecl:blu byr:1944 eyr:2021 pid:093154719";
            answers = &[None, Some(Unsigned(4))];
        }
        actual_answers = unsigned![202, 137];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use nom::{
        branch::alt,
        bytes::complete::{is_not, tag, take_until, take_while_m_n},
        character::complete::{line_ending, space1},
        combinator::{all_consuming, map},
        multi::separated_list1,
        sequence::{pair, preceded, separated_pair},
    };
    use std::collections::HashMap;
    use strum::{EnumIter, EnumString, IntoEnumIterator};

    /// Possible passport fields, which can be parsed from text input.
    #[derive(PartialEq, Eq, Hash, Debug, EnumString, EnumIter)]
    pub enum PassportField {
        /// Birth year.
        #[strum(serialize = "byr")]
        BirthYear,
        /// Issue year.
        #[strum(serialize = "iyr")]
        IssueYear,
        /// Exppiration year.
        #[strum(serialize = "eyr")]
        ExpirationYear,
        /// Person's height.
        #[strum(serialize = "hgt")]
        Height,
        /// Person's hair color.
        #[strum(serialize = "hcl")]
        HairColor,
        /// Person's eye color.
        #[strum(serialize = "ecl")]
        EyeColor,
        /// Passport ID.
        #[strum(serialize = "pid")]
        PassportId,
        /// Country ID of the passport.
        #[strum(serialize = "cid")]
        CountryId,
    }
    impl PassportField {
        /// Returns whether a field value is valid or not given the field type.
        fn validate(&self, value: &str) -> bool {
            use PassportField::*;
            match self {
                BirthYear => match value.parse::<u32>() {
                    Ok(b) => (1920..=2002).contains(&b),
                    Err(_) => false,
                },
                IssueYear => match value.parse::<u32>() {
                    Ok(b) => (2010..=2020).contains(&b),
                    Err(_) => false,
                },
                ExpirationYear => match value.parse::<u32>() {
                    Ok(b) => (2020..=2030).contains(&b),
                    Err(_) => false,
                },
                Height => {
                    let res: NomParseResult<&str, (u32, &str)> = all_consuming(pair(
                        nom::character::complete::u32,
                        alt((tag("cm"), tag("in"))),
                    ))(value);
                    match res {
                        Ok((_, (h, u))) => match u {
                            "cm" => (150..=193).contains(&h),
                            _ => (59..=76).contains(&h),
                        },
                        Err(_) => false,
                    }
                }
                HairColor => {
                    let res: NomParseResult<&str, &str> = all_consuming(preceded(
                        tag("#"),
                        take_while_m_n(6, 6, |c: char| c.is_ascii_hexdigit()),
                    ))(value);
                    res.is_ok()
                }
                EyeColor => {
                    let res: NomParseResult<&str, &str> = all_consuming(alt((
                        tag("amb"),
                        tag("blu"),
                        tag("brn"),
                        tag("gry"),
                        tag("grn"),
                        tag("hzl"),
                        tag("oth"),
                    )))(value);
                    res.is_ok()
                }
                PassportId => {
                    let res: NomParseResult<&str, &str> =
                        all_consuming(take_while_m_n(9, 9, |c: char| c.is_ascii_digit()))(value);
                    res.is_ok()
                }
                CountryId => true,
            }
        }
    }
    impl Parsable<'_> for PassportField {
        fn parser(input: &str) -> NomParseResult<&str, Self>
        where
            Self: Sized,
        {
            map(take_until(":"), |s: &str| {
                s.parse()
                    .unwrap_or_else(|_| panic!("Unknown passport field: {s}"))
            })(input)
        }
    }

    /// Hash map from a passport field to its value.
    type PassportFieldMap = HashMap<PassportField, String>;

    /// Behavior for a particular part of the problem.
    pub trait Part {
        /// Validates whether a passport is valid.
        fn validate(field_map: &PassportFieldMap) -> bool;
    }

    /// Behavior for part one.
    pub struct PartOne {}
    impl Part for PartOne {
        fn validate(field_map: &PassportFieldMap) -> bool {
            PassportField::iter()
                .all(|field| field_map.contains_key(&field) || field == PassportField::CountryId)
        }
    }

    /// Behavior for part two.
    pub struct PartTwo {}
    impl Part for PartTwo {
        fn validate(field_map: &PassportFieldMap) -> bool {
            PassportField::iter().all(|field| match field_map.get(&field) {
                Some(v) => field.validate(v),
                None => field == PassportField::CountryId,
            })
        }
    }

    /// A passport with its fields, which can be parsed from text input.
    struct Passport {
        /// Map from fields to their values.
        field_map: PassportFieldMap,
    }
    impl Parsable<'_> for Passport {
        fn parser(input: &str) -> NomParseResult<&str, Self>
        where
            Self: Sized,
        {
            map(
                separated_list1(
                    alt((space1, line_ending)),
                    separated_pair(PassportField::parser, tag(":"), is_not(" \t\n\r")),
                ),
                |vec| Self {
                    field_map: vec.into_iter().map(|(k, v)| (k, v.to_string())).collect(),
                },
            )(input)
        }
    }
    impl Passport {
        /// Validates the passport for a particular part of the problem.
        pub fn validate<P: Part>(&self) -> bool {
            P::validate(&self.field_map)
        }
    }

    /// List of passports, which can be parsed from text input.
    pub struct PassportList {
        /// List of passports.
        passports: Vec<Passport>,
    }
    impl PassportList {
        /// Parses the list from text input.
        pub fn from_str(input: &str) -> AocResult<Self> {
            Ok(Self {
                passports: Passport::gather(input.split("\n\n"))?,
            })
        }

        /// Counts the number of passports in the list that are valid for a particular part of the problem.
        pub fn count_valid<P: Part>(&self) -> u64 {
            self.passports.iter().filter_count(|p| p.validate::<P>())
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 4,
    name: "Passport Processing",
    preprocessor: Some(|input| Ok(Box::new(PassportList::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Processing
            Ok(input
                .expect_data::<PassportList>()?
                .count_valid::<PartOne>()
                .into())
        },
        // Part two
        |input| {
            // Processing
            Ok(input
                .expect_data::<PassportList>()?
                .count_valid::<PartTwo>()
                .into())
        },
    ],
};
