use aoc::prelude::*;
use serde_json::Value;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_tests;
    use Answer::Signed;

    solution_tests! {
        example {
            input = "[1,2,3]";
            answers = vec![6i64, 6].answer_vec();
        }
        example {
            input = r#"{"a":2,"b":4}"#;
            answers = vec![6i64, 6].answer_vec();
        }
        example {
            input = "[[[3]]]";
            answers = vec![3i64, 3].answer_vec();
        }
        example {
            input = r#"{"a":{"b":4},"c":-1}"#;
            answers = vec![3i64, 3].answer_vec();
        }
        example {
            input = r#"{"a":[-1,1]}"#;
            answers = vec![0i64, 0].answer_vec();
        }
        example {
            input = r#"[-1,{"a":1}]"#;
            answers = vec![0i64, 0].answer_vec();
        }
        example {
            input = "[]";
            answers = vec![0i64, 0].answer_vec();
        }
        example {
            input = "{}";
            answers = vec![0i64, 0].answer_vec();
        }
        example {
            input = r#"[1,{"c":"red","b":2},3]"#;
            answers = vec![6i64, 4].answer_vec();
        }
        example {
            input = r#"{"d":"red","e":[1,2,3,4],"f":5}"#;
            answers = vec![15i64, 0].answer_vec();
        }
        example {
            input = r#"[1,"red",5]"#;
            answers = vec![6i64, 6].answer_vec();
        }
        actual_answers = vec![Signed(191164), Signed(87842)];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;

    /// Parses JSON from text input and returns the root JSON value.
    pub fn parse_json(s: &str) -> AocResult<Value> {
        serde_json::from_str(s)
            .map_err(|e| AocError::InvalidInput(format!("Invalid JSON: {e}").into()))
    }

    /// Behavior specific to a particular part of the problem.
    pub trait Part {
        /// Determines whether a JSON value is valid and should be included in the sum.
        fn valid_value(_value: &Value) -> bool {
            true
        }

        /// Adds up all the numbers appearing in an iterator of JSON values, counting only those that valid.
        fn value_sums<'a>(values: impl Iterator<Item = &'a Value>) -> i64
        where
            Self: Sized,
        {
            values
                .filter(|v| Self::valid_value(v))
                .map(|v| v.sum_numbers::<Self>())
                .sum()
        }
    }

    /// Behavior for part one.
    pub struct PartOne;
    impl Part for PartOne {}

    /// Behavior for part two.
    pub struct PartTwo;
    impl Part for PartTwo {
        fn valid_value(value: &Value) -> bool {
            match value {
                Value::Object(m) => !m
                    .values()
                    .any(|v| matches!(v, Value::String(s) if s == "red")),
                _ => true,
            }
        }
    }

    /// Capability to determine the sum of numbers contained in a particular JSON value.
    pub trait SumNumbers {
        /// Calculates the number sum of the JSON value recursively.
        fn sum_numbers<P: Part>(&self) -> i64;
    }
    impl SumNumbers for Value {
        fn sum_numbers<P: Part>(&self) -> i64 {
            if P::valid_value(self) {
                match self {
                    Value::Number(n) => n.as_i64().unwrap_or(0),
                    Value::Array(v) => P::value_sums(v.iter()),
                    Value::Object(m) => P::value_sums(m.values()),
                    _ => 0,
                }
            } else {
                0
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 12,
    name: "JSAbacusFramework.io",
    preprocessor: Some(|input| Ok(Box::new(parse_json(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Value>()?
                .sum_numbers::<PartOne>()
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Value>()?
                .sum_numbers::<PartTwo>()
                .into())
        },
    ],
};
