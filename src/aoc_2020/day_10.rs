use super::super::aoc::{
    AocError,
    Parseable,
    Solution,
};

#[cfg(test)]
mod tests{
    use super::*;
    use crate::solution_test;

    solution_test! {
        "28
33
18
42
31
14
46
20
48
47
24
23
49
45
19
38
39
11
1
32
25
35
8
17
7
9
4
2
34
10
3",
        vec![220],
        vec![2100]
    }
}

pub const SOLUTION: Solution = Solution {
    day: 10,
    name: "Adapter Array",
    solver: |input| {
        // Generation
        let joltages = {
            // Get joltages 
            let mut j = u32::gather(input.lines())?;
            // Add the outlet
            j.push(0);
            // Add the device
            j.push(j.iter().max().unwrap() + 3);
            j.sort();
            j
        };

        // Processing
        let diffs: Vec<u32> = joltages.windows(2).map(|w| w[1] - w[0]).collect();
        // Verify that no differences are above 3
        if diffs.iter().any(|d| *d > 3) {
            return Err(AocError::Process("Adaptors cannot be chained together due to a gap of over 3 jolts".to_string()));
        }
        // Now get the required diffs
        let count_diffs = |n| {
            diffs.iter().filter(|d| **d == n).count() as u64
        };
        let answers = vec![
            count_diffs(1) * count_diffs(3),
        ];

        Ok(answers)
    },
};
