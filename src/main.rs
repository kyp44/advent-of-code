//! # Advent of Code
//!
//! These are my solutions to the [Advent of Code](https://adventofcode.com/) problems in Rust.
//! I started this to help me learn Rust, but now the problems are just fun to solve and also help keep my Rust skills sharp!
//!
//! All the code is documented to some extent, including general utilities in the [`aoc`] module
//! that are used in multiple solutions.
//!
//! Also see the LaTeX notes for problems that required more analysis.
//! The document is in the `notes` directory and includes a `Makefile`.
#![feature(hash_set_entry)]
#![feature(type_alias_impl_trait)]
#![feature(let_chains)]
#![feature(is_some_and)]
#![feature(step_trait)]
#![feature(once_cell)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

mod aoc_2015;
mod aoc_2020;
mod aoc_2021;

use aoc::AocError;
use clap::Parser;
use colored::Colorize;
use itertools::Itertools;

/// Run the Advent of Code solution for a particular year and day.
#[derive(Parser)]
#[command(name = "Advent of Code Solutions", author, version)]
struct Args {
    /// List the implemented solutions.
    #[arg(short, long)]
    list: bool,
    /// Year of the problem solution to run.
    #[arg(name = "YEAR", required_unless_present("list"))]
    year: Option<u32>,
    /// Day of the problem solution to run (1-25).
    #[arg(name = "DAY", required_unless_present("list"))]
    day: Option<u32>,
}

/// Runs the program, of course.
fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let cli = Args::parse();

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
                        "{}\n{}",
                        format!("{year}").bold().underline(),
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
