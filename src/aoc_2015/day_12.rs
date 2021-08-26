use serde_json::Value;

use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Signed;

    solution_test! {
    vec![Signed(191164), Signed(87842)],
    "[1,2,3]",
    vec![6i64, 6].answer_vec(),
    r#"{"a":2,"b":4}"#,
    vec![6i64, 6].answer_vec(),
    "[[[3]]]",
    vec![3i64, 3].answer_vec(),
    r#"{"a":{"b":4},"c":-1}"#,
    vec![3i64, 3].answer_vec(),
    r#"{"a":[-1,1]}"#,
    vec![0i64, 0].answer_vec(),
    r#"[-1,{"a":1}]"#,
    vec![0i64, 0].answer_vec(),
    "[]",
    vec![0i64, 0].answer_vec(),
    "{}",
    vec![0i64, 0].answer_vec(),
    r#"[1,{"c":"red","b":2},3]"#,
    vec![6i64, 4].answer_vec(),
    r#"{"d":"red","e":[1,2,3,4],"f":5}"#,
    vec![15i64, 0].answer_vec(),
    r#"[1,"red",5]"#,
    vec![6i64, 6].answer_vec()
    }
}

fn parse_json(s: &str) -> AocResult<Value> {
    serde_json::from_str(s)
        .map_err(|e| AocError::InvalidInput(format!("Invalid JSON: {}", e).into()))
}

trait Part {
    fn valid_value(_value: &Value) -> bool {
        true
    }

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
struct PartA;
impl Part for PartA {}
struct PartB;
impl Part for PartB {
    fn valid_value(value: &Value) -> bool {
        match value {
            Value::Object(m) => !m
                .values()
                .any(|v| matches!(v, Value::String(s) if s == "red")),
            _ => true,
        }
    }
}

trait SumNumbers {
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

pub const SOLUTION: Solution = Solution {
    day: 12,
    name: "JSAbacusFramework.io",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let data = parse_json(input)?;

            // Process
            Ok(data.sum_numbers::<PartA>().into())
        },
        // Part b)
        |input| {
            // Generation
            let data = parse_json(input)?;

            // Process
            Ok(data.sum_numbers::<PartB>().into())
        },
    ],
};
