use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aoc::solution_test;
    use Answer::Unsigned;

    solution_test! {
        vec![Unsigned(6335), Unsigned(3392)],
        "abc

a
b
c

ab
ac

a
a
a
a

b
",
        vec![11u64, 6].answer_vec()
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use std::collections::HashSet;

    /// Set of questions.
    type Questions = HashSet<char>;

    /// A set of questions with "yes" answers for a single person.
    ///
    /// This can be parsed from text input.
    struct PersonQuestions {
        /// List of questions.
        questions: Questions,
    }
    impl From<&str> for PersonQuestions {
        fn from(value: &str) -> Self {
            Self {
                questions: value.trim().chars().collect(),
            }
        }
    }

    /// Group of people who answered questions, which can be parsed from text input.
    pub struct Group {
        /// Questions for everyone in the group.
        people_questions: Vec<PersonQuestions>,
    }
    impl From<&str> for Group {
        fn from(value: &str) -> Self {
            Self {
                people_questions: value.lines().map(PersonQuestions::from).collect(),
            }
        }
    }
    impl Group {
        /// Returns the set of questions for which anyone in the group answered "yes".
        pub fn any_questions(&self) -> Questions {
            self.reduce_questions(|a, b| a.union(&b).copied().collect())
        }

        /// Returns the set of questions for which everyone in the group answered "yes".
        pub fn all_questions(&self) -> Questions {
            self.reduce_questions(|a, b| a.intersection(&b).copied().collect())
        }

        /// Reduces the people's question sets using a set combinator.
        fn reduce_questions(&self, reducer: fn(Questions, Questions) -> Questions) -> Questions {
            self.people_questions
                .iter()
                .map(|pq| pq.questions.clone())
                .reduce(reducer)
                .unwrap()
        }
    }

    /// Solves a problem by summing the number of questions for each group.
    pub fn solve(input: &SolverInput, group_f: fn(&Group) -> Questions) -> AocResult<Answer> {
        Ok(Answer::Unsigned(
            input
                .expect_data::<Vec<Group>>()?
                .iter()
                .map(|group| group_f(group).len())
                .sum::<usize>()
                .try_into()
                .unwrap(),
        ))
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 6,
    name: "Custom Customs",
    preprocessor: Some(|input| {
        Ok(Box::new(input.split("\n\n").map(Group::from).collect::<Vec<Group>>()).into())
    }),
    solvers: &[
        // Part one
        |input| {
            // Process
            solve(input, Group::any_questions)
        },
        // Part two
        |input| {
            // Process
            solve(input, Group::all_questions)
        },
    ],
};
