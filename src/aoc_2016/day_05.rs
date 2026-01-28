use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "abc";
            answers = string!["18f47a30", "05ace8e3"];
        }
        actual_answers = string!["801b56a7"];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use derive_more::From;
    use md5::Digest;

    #[derive(From)]
    struct Md5Hash(Digest);
    impl Md5Hash {
        pub fn password_char(&self) -> Option<char> {
            let hd = &self.0.0;
            // Test whether the first 5 hex digits are zero
            (hd[0] == 0 && hd[1] == 0 && hd[2] < 16)
                .then(|| char::from_digit(u32::from(hd[2]) & 0x0F, 16).unwrap())
        }
    }

    pub fn find_password(door_id: &str) -> String {
        let mut index = 0u64;
        (0..8)
            .map(|_| {
                loop {
                    let hash: Md5Hash = md5::compute(format!("{door_id}{index}")).into();
                    index += 1;
                    if let Some(c) = hash.password_char() {
                        break c;
                    }
                }
            })
            .collect()
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 5,
    name: "How About a Nice Game of Chess?",
    preprocessor: None,
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(find_password(input.expect_text()?.trim()).into())
        },
    ],
};
