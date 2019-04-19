#[macro_use]
extern crate lazy_static;

mod common;
mod normal;
mod reverse;

use clap::{App, ArgMatches, SubCommand};

// runs the program
fn run_app(matches: &ArgMatches) -> common::Result<()> {
    match matches.subcommand() {
        ("reverse", Some(sub_matches)) => reverse::run(sub_matches),
        _ => normal::run(matches),
    }
}

fn main() {
    // CLI Arguments parsing
    let matches = App::new("get621")
        .version("1.2.0")
        .author("nasso <nassomails ~ at ~ gmail {dot} com>")
        // default command
        .about("E621/926 command line tool")
        .args(&normal::args())
        // reverse subcommand
        .subcommand(
            SubCommand::with_name("reverse")
                .about("E621/926 reverse searching utils")
                .args(&reverse::args()),
        )
        .get_matches();

    ::std::process::exit(match run_app(&matches) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    })
}
