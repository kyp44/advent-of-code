mod aoc;
mod aoc_2020;

use aoc::AocError;
use structopt::StructOpt;

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

fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let cli = Cli::from_args();

    // Get solution or produce errors if it is not implemented
    let all_year_solutions = vec![&aoc_2020::YEAR_SOLUTIONS];
    let year_solutions = all_year_solutions.iter()
        .find(|ys| ys.year == cli.year).ok_or(AocError::NoYear(cli.year))?;
    let solution = year_solutions.get_day(cli.day).ok_or(AocError::NoDay(cli.day))?;

    // Run the solution
    solution.run(year_solutions.year)?;

    Ok(())
}
