mod aoc_2020;

use clap::{crate_version, value_t_or_exit, App, Arg};

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

    let year = value_t_or_exit!(matches, "YEAR", u32);
    let day = value_t_or_exit!(matches, "DAY", u32);
    println!("Year: {} Day: {}", year, day);
}
