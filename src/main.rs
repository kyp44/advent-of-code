//! # Advent of Code
//!
//! These are my solutions to the [Advent of Code](https://adventofcode.com/) problems in Rust.
//! I am working through these to help me learn Rust and because the problems are fun to solve!
//!
//! All the code is documented to some extent, including general utilities in the [aoc] module
//! that are used in multiple solutions.
//!
//! Also see the LaTeX notes for problems that required more analysis.
//! The document is in the `notes` directory and includes a `Makefile`.
#![feature(hash_set_entry)]
#![feature(type_alias_impl_trait)]
#![feature(slice_pattern)]
#![feature(let_chains)]
#![feature(is_some_and)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

#[macro_use]
extern crate derive_new;

#[macro_use]
extern crate lazy_static;

mod aoc;
mod aoc_2015;
mod aoc_2020;
mod aoc_2021;

use aoc::AocError;
use itertools::Itertools;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Advent of Code Solutions",
    author = "Dan Whitman <dwhitman44@gmail.com>",
    about = "Run the Advent of Code solution for a particular year and day."
)]

/// Command line arguments.
struct Cli {
    /// List the implemented solutions
    #[structopt(short, long)]
    list: bool,
    /// Year to run
    #[structopt(name = "YEAR", required_unless("list"))]
    year: Option<u32>,
    /// Day to run (1-25)
    #[structopt(name = "DAY", required_unless("list"))]
    day: Option<u32>,
}

fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let cli = Cli::from_args();

    let all_year_solutions = vec![
        &aoc_2015::YEAR_SOLUTIONS,
        &aoc_2020::YEAR_SOLUTIONS,
        &aoc_2021::YEAR_SOLUTIONS,
    ];

    if cli.list {
        // List all implemented solutions
        println!(
            "{}",
            all_year_solutions
                .into_iter()
                .map(|year_solutions| {
                    let year = year_solutions.year;
                    format!(
                        "{}\n{}\n{}",
                        year,
                        year.to_string().chars().map(|_| '=').collect::<String>(),
                        year_solutions.solution_list(),
                    )
                })
                .join("\n\n")
        );
    } else {
        // Get solution or produce errors if it is not implemented
        let year = cli.year.unwrap();
        let day = cli.day.unwrap();
        let year_solutions = all_year_solutions
            .iter()
            .find(|ys| ys.year == year)
            .ok_or(AocError::NoYear(year))?;
        let day_range = 1..=25;
        if !day_range.contains(&day) {
            return Err(AocError::DayRange(day, day_range).into());
        }
        let solution = year_solutions.get_day(day).ok_or(AocError::NoDay(day))?;

        // Run the solution
        solution.run_and_print(year_solutions.year)?;
    }

    Ok(())
}
