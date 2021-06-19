mod aoc;
mod aoc_2020;

use aoc::AocError;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Advent of Code Solutions", author = "Dan Whitman <dwhitman44@gmail.com>", about = "Run the Advent of Code solution for a particular year and day.")]
struct Cli {
    /// List the implemented solutions
    #[structopt(short, long)]
    list: bool,
    /// Year to run (2015-2020)
    #[structopt(name = "YEAR", required_unless("list"))]
    year: Option<u32>,
    /// Day to run (1-25)
    #[structopt(name = "DAY", required_unless("list"))]
    day: Option<u32>,
}

fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let cli = Cli::from_args();

    let all_year_solutions = vec![&aoc_2020::YEAR_SOLUTIONS];
    
    if cli.list {
        // List all implemented solutions
        for year_solutions in all_year_solutions {
            year_solutions.print_solution_list();
        }
    } else {
        // Get solution or produce errors if it is not implemented
        let year = cli.year.unwrap();
        let day = cli.day.unwrap();
        let year_solutions = all_year_solutions.iter()
            .find(|ys| ys.year == year).ok_or(AocError::NoYear(year))?;
        let solution = year_solutions.get_day(day).ok_or(AocError::NoDay(day))?;

        // Run the solution
        solution.run(year_solutions.year)?;
    }

    Ok(())
}
