mod common;
mod normal;
mod pool;
mod reverse;

use clap::{crate_version, App, ArgMatches};

// runs the program
async fn run(matches: &ArgMatches<'_>) -> common::Result<()> {
    match matches.subcommand() {
        ("pool", Some(sub_matches)) => pool::run(sub_matches).await,
        ("reverse", Some(sub_matches)) => reverse::run(sub_matches).await,
        _ => normal::run(matches).await,
    }
}

#[tokio::main]
async fn main() {
    // CLI Arguments parsing
    let matches = App::new("get621")
        .version(&crate_version!()[..])
        .author("nasso <nassomails ~ at ~ gmail {dot} com>")
        // default command
        .about("E621/926 command line tool")
        .args(&normal::args())
        .subcommand(pool::subcommand())
        .subcommand(reverse::subcommand())
        .get_matches();

    ::std::process::exit(match run(&matches).await {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    })
}
