mod aoc;
mod aoc_2020;

use aoc::AocError;
use std::fs;
use structopt::StructOpt;
use anyhow::{Context, Result};

#[derive(Debug, StructOpt)]
#[structopt(name = "Advent of Code Solutions", author = "Dan Whitman <dwhitman44@gmail.com>", about = "Run the Advent of Code solution for a particular year and day.")]
struct Cli {
    /// Year to run (2015-2020)
    #[structopt(name = "YEAR")]
    year: u32,
    /// Day to run (1-25)
    #[structopt(name = "DAY")]
    day: u32,
}

fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::from_args();

    // Read input for the problem
    let input_path = format!("input/{}/day_{:02}.txt", cli.year, cli.day);
    let input_content = fs::read_to_string(&input_path)
        .with_context(|| format!("Could not read input file {}", input_path))?;
    let input = input_content.trim_end();

    // Dispatch solution function
    // This should really be done in a better way, ideally using macro tags for the functions
    let result = match cli.year {
        2020 => {
            match cli.day {
                2 => aoc_2020::password_philosophy(input),
                _ => Err(AocError::NoDay(cli.day)),
            }
        },
        _ => Err(AocError::NoYear(cli.year)),
    }.with_context(|| "Problem when running the solution")?;

    Ok(())
}
