mod aoc_2020;

use clap::{crate_version, value_t, App, Arg};

fn main() {
    let matches = App::new("Advent of Code Solutions")
        .version(crate_version!())
        .author("Dan Whitman <dwhitman44@gmail.com>")
        .about("Run the Advent of Code solution for a particular year and day.")
        .arg(Arg::with_name("YEAR")
             .help("Year to run")
             .required(true))
        .arg(Arg::with_name("DAY")
             .help("Day to run (typically 1-25)")
             .required(true))
        .get_matches();

    let year = value_t!(matches, "YEAR", u32).unwrap_or_else(|e| e.exit());
    let day = value_t!(matches, "DAY", u32).unwrap_or_else(|e| e.exit());
    println!("Year: {} Day: {}", year, day);
}
