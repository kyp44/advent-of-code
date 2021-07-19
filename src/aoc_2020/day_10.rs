use crate::aoc::AocResult;

use super::super::aoc::{AocError, FilterCount, Parseable, Solution};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;

    solution_test! {
    vec![2100, 16198260678656],
    "16
10
15
5
1
11
7
19
6
12
4",
    vec![Some(35), Some(8)],
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
    vec![Some(220), Some(19208)]
    }
}

fn parse_joltages(input: &str) -> AocResult<Vec<u32>> {
    // Get joltages
    let mut j = u32::gather(input.lines())?;
    // Add the outlet
    j.push(0);
    // Add the device
    j.push(j.iter().max().unwrap() + 3);
    // Remove duplicates and sort
    j.dedup();
    j.sort_unstable();
    Ok(j)
}

pub const SOLUTION: Solution = Solution {
    day: 10,
    name: "Adapter Array",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let joltages = parse_joltages(input)?;

            // Processing
            let diffs: Vec<u32> = joltages.windows(2).map(|w| w[1] - w[0]).collect();
            // Verify that no differences are above 3
            if diffs.iter().any(|d| *d > 3) {
                return Err(AocError::Process(
                    "Adaptors cannot be chained together due to a gap of over 3 jolts".to_string(),
                ));
            }
            // Now get the required diffs
            let count_diffs = |n| -> u64 { diffs.iter().filter_count(|d| **d == n) };
            Ok(count_diffs(1) * count_diffs(3))
        },
        // Part b)
        |input| {
            // Generation
            let joltages = parse_joltages(input)?;

            // Processing
            // For each adapter we store the number of variations ahead if we were to just start with that
            // joltage.
            let mut variations: HashMap<u32, u64> = HashMap::new();
            let mut last_var = 1;
            // The algorithm here works work backwards just because it's more natural to take slices
            // forward rather than backward.
            for (i, v) in joltages.iter().enumerate().rev() {
                // Each new number of variations is then the sum of any potential number
                // of variations if there are adapters with any of the next three consectuive
                // joltages, or the last variation if the the next gap is 3 jolts.
                let var = std::cmp::max(
                    joltages[i + 1..]
                        .iter()
                        .take_while(|vp| **vp <= v + 3)
                        .map(|vp| variations[vp])
                        .sum(),
                    last_var,
                );
                variations.insert(*v, var);
                last_var = var;
                //println!("{} {} {}", i, v, var);
            }
            Ok(last_var)
        },
    ],
};
