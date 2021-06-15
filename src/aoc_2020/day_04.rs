use super::super::aoc::{
    ParseResult,
    Solution,
    CountFilter
};
use nom::{
    Finish,
    branch::alt,
    bytes::complete::{is_not, tag, take_while_m_n},
    character::{is_digit, is_hex_digit},
    character::complete::{digit1, line_ending, space0, space1},
    combinator::{all_consuming, map},
    error::context,
    multi::separated_list1,
    sequence::{pair, separated_pair, tuple, preceded},
};
use std::{collections::HashMap, iter::FromIterator};
use strum::IntoEnumIterator;
use strum_macros::{EnumString, EnumIter};

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn initial_example() {
        let input = "ecl:gry pid:860033327 eyr:2020 hcl:#fffffd
byr:1937 iyr:2017 cid:147 hgt:183cm

iyr:2013 ecl:amb cid:350 eyr:2023 pid:028048884
hcl:#cfa07d byr:1929

hcl:#ae17e1 iyr:2013
eyr:2024
ecl:brn pid:760753108 byr:1931
hgt:179cm

hcl:#cfa07d eyr:2025 pid:166559648
iyr:2011 ecl:brn hgt:59in";
        
        assert_eq!((SOLUTION.solver)(input).unwrap(), vec![2,2]);
    }

    #[test]
    fn invalid_passports() {
        let input = "eyr:1972 cid:100
hcl:#18171d ecl:amb hgt:170 pid:186cm iyr:2018 byr:1926

iyr:2019
hcl:#602927 eyr:1967 hgt:170cm
ecl:grn pid:012533040 byr:1946

hcl:dab227 iyr:2012
ecl:brn hgt:182cm pid:021572410 eyr:2020 byr:1992 cid:277

hgt:59cm ecl:zzz
eyr:2038 hcl:74454a iyr:2023
pid:3556412378 byr:2007";

        assert_eq!((SOLUTION.solver)(input).unwrap(), vec![4,0]);
    }

    #[test]
    fn valid_passports() {
        let input = "pid:087499704 hgt:74in ecl:grn iyr:2012 eyr:2030 byr:1980
hcl:#623a2f

eyr:2029 ecl:blu cid:129 byr:1989
iyr:2014 pid:896056539 hcl:#a97842 hgt:165cm

hcl:#888785
hgt:164cm byr:2001 iyr:2015 cid:88
pid:545766238 ecl:hzl
eyr:2022

iyr:2010 hgt:158cm hcl:#b6652a ecl:blu byr:1944 eyr:2021 pid:093154719";
        
        assert_eq!((SOLUTION.solver)(input).unwrap(), vec![4,4]);
    }

    #[test]
    #[ignore]
    fn actual() {
        assert_eq!(SOLUTION.run(super::super::YEAR_SOLUTIONS.year).unwrap(), vec![202, 137]);
    }    
}

#[derive(PartialEq, Eq, Hash, Debug, EnumString, EnumIter)]
enum PassportField {
    #[strum(serialize = "byr")]
    BirthYear,
    #[strum(serialize = "iyr")]
    IssueYear,
    #[strum(serialize = "eyr")]
    ExpirationYear,
    #[strum(serialize = "hgt")]
    Height,
    #[strum(serialize = "hcl")]
    HairColor,
    #[strum(serialize = "ecl")]
    EyeColor,
    #[strum(serialize = "pid")]
    PassportId,
    #[strum(serialize = "cid")]
    CountryId,
}

impl PassportField {
    fn valid(&self, value: &str) -> bool {
        use PassportField::*;
        match self {
            BirthYear => {
                match value.parse::<u32>() {
                    Ok(b) => 1920 <= b && b <= 2002,
                    Err(_) => false
                }
            },
            IssueYear => {
                match value.parse::<u32>() {
                    Ok(b) => 2010 <= b && b <= 2020,
                    Err(_) => false
                }
            },
            ExpirationYear => {
                match value.parse::<u32>() {
                    Ok(b) => 2020 <= b && b <= 2030,
                    Err(_) => false
                }
            },
            Height => {
                let res: ParseResult<(&str, &str)> = all_consuming(
                    pair(
                        digit1,
                        alt((tag("cm"), tag("in")))

                    )
                )(value);
                match res {
                    Ok((_, (h, u))) => {
                        let h = h.parse::<u32>().unwrap();
                        match u {
                            "cm" => 150 <= h && h <= 193,
                            _ => 59 <= h && h <= 76,
                        }
                    },
                    Err(_) => false,
                }
            },
            HairColor => {
                let res: ParseResult<&str> = all_consuming(
                    preceded(
                        tag("#"),
                        take_while_m_n(6, 6, |c: char| c.is_ascii() && is_hex_digit(c as u8)),
                    )
                )(value);
                res.is_ok()
            },
            EyeColor => {
                let res: ParseResult<&str> = all_consuming(
                    alt((
                        tag("amb"),
                        tag("blu"),
                        tag("brn"),
                        tag("gry"),
                        tag("grn"),
                        tag("hzl"),
                        tag("oth"),
                    ))
                )(value);
                res.is_ok()
            },
            PassportId => {
                let res: ParseResult<&str> = all_consuming(
                    take_while_m_n(9, 9, |c: char| c.is_ascii() && is_digit(c as u8))
                )(value);
                res.is_ok()
            },
            CountryId => true,
        }
    }
}

type Passport<'a> = HashMap<PassportField, &'a str>;

// Note that this could not be done as an impl of the Parseable trait
// beause of annoying lifetime issues that apparently have no solution in Rust.
// It also could not be implemented as a normal method on Passport since
// this is a foreign type.
fn parse_passport<'a>(input: &'a str) -> ParseResult<Passport<'a>> {
    context(
        "passport data",
        map(
            separated_list1(
                alt((pair(space0, line_ending), pair(space1, space0))),
                separated_pair(
                    is_not(": \n\r"),
                    tag(":"),
                    is_not(" \t\n\r"),
                )
            ),
            |v: Vec<(&str, &str)>| {
                HashMap::from_iter(v.iter().filter_map(|(k, v)| {
                    let pfr: Result<PassportField, strum::ParseError> = k.parse();
                    match pfr {
                        Ok(pf) => Some((pf, *v)),
                        Err(_) => None
                    } 
                }))
            }
        )
    )(input)
}

// Note that we can't make this a method of Passport because it's
// A foreign type. We could make a trait but nah.
fn passport_valid_part_a(passport: &Passport) -> bool {
    let mut valid = true;
    for field in PassportField::iter() {
        if !passport.contains_key(&field) && field != PassportField::CountryId {
            valid = false;
            break;
        }
    }
    valid
}

// Part b
fn passport_valid_part_b(passport: &Passport) -> bool {
    let mut valid = true;
    for field in PassportField::iter() {
        valid = match passport.get(&field) {
            Some(v) => {
                field.valid(v)
            },
            None => field == PassportField::CountryId,
        };
        if !valid {
            break;
        }
    }
    valid
}

pub const SOLUTION: Solution = Solution {
    day: 4,
    name: "Passport Processing",
    solver: |input| {
        // Generation
        let passports = all_consuming(
            separated_list1(
                tuple((space0, line_ending, space0, line_ending)),
                parse_passport,
            )
        )(input.trim_end()).finish().map(|(_, pd)| pd)?;

        // Processing
        let mut answers = vec![];
        answers.push(passports.iter().filter_count(|p| passport_valid_part_a(p)));
        answers.push(passports.iter().filter_count(|p| passport_valid_part_b(p)));

        Ok(answers)
    }
};