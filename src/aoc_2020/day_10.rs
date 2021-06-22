use super::super::aoc::{AocError, Parseable, Solution};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let input = "16
10
15
5
1
11
7
19
6
12
4";
        assert_eq!((SOLUTION.solver)(input).unwrap(), vec![35, 8]);

        let input = "28
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
3";
        assert_eq!((SOLUTION.solver)(input).unwrap(), vec![220, 19208]);
    }

    #[test]
    #[ignore]
    fn actual() {
        assert_eq!(
            SOLUTION.run(super::super::YEAR_SOLUTIONS.year).unwrap(),
            vec![2100, 16198260678656]
        );
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
            // Remove duplicates and sort
            j.dedup();
            j.sort_unstable();
            j
        };

        // Processing
        // Part a)
        let diffs: Vec<u32> = joltages.windows(2).map(|w| w[1] - w[0]).collect();
        // Verify that no differences are above 3
        if diffs.iter().any(|d| *d > 3) {
            return Err(AocError::Process(
                "Adaptors cannot be chained together due to a gap of over 3 jolts".to_string(),
            ));
        }
        // Now get the required diffs
        let count_diffs = |n| diffs.iter().filter(|d| **d == n).count() as u64;
        let mut answers = vec![count_diffs(1) * count_diffs(3)];

        // Part b)
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
            println!("{} {} {}", i, v, var);
        }
        answers.push(last_var);

        Ok(answers)
    },
};
