mod aoc;
mod aoc_2020;
use ansi_term::Color;

use aoc::AocError;
use std::fs;
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

// Annoying that we have to do this but there seems to be no way around it
macro_rules! unwrap_or_error {
    ( $s:expr, $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{}\t{}:\n\t{}", Color::Red.bold().paint("error:"), $s, e);
                return Err(())
            },
        }
    }
}

fn run() -> Result<(), ()> {
    // Parse command line arguments
    let cli = Cli::from_args();

    // Read input for the problem
    let input_path = format!("input/{}/day_{:02}.txt", cli.year, cli.day);
    let input = unwrap_or_error!(
        format!("Could not read input file {}", input_path),
        fs::read_to_string(&input_path)
    );

    // Dispatch solution function
    // This should really be done in a better way, ideally using macro tags for the functions
    let result = unwrap_or_error!(
        "Problem when running the solution",
        match cli.year {
            2020 => {
                match cli.day {
                    2 => aoc_2020::password_philosophy(&input),
                    _ => Err(AocError::NoDay(cli.day)),
                }
            },
            _ => Err(AocError::NoYear(cli.year)),
        }
    );

    println!("Result: {}", result);

    Ok(())
}

fn main() {
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(_) => 1,
    })
}
