extern crate clap;

use std::str::FromStr;

use clap::{Arg, App, ArgMatches};

use get621::{Get621, Error};

fn valid_parse<T: FromStr>(v: &str, emsg: &str) -> Result<(), String> {
	match v.parse::<T>() {
		Ok(_) => Ok(()),
		Err(_) => Err(String::from(emsg))
	}
}

fn run_app(matches: ArgMatches) -> Result<(), String> {
	// Get args
	let limit = matches.value_of("limit").unwrap().parse().unwrap();
	let verbose = matches.is_present("verbose");
	let tags = matches.values_of("tags").map_or_else(|| Vec::new(), |v| v.collect::<Vec<_>>());
	
	let res = Get621::init()
		.and_then(|g6| g6.list(&tags, limit));
	
	// Get posts
	match res {
		Ok(posts) => {
			if verbose {
				println!(
					"{}",
					posts.iter()
					     .map(|p| p.to_string())
					     .collect::<Vec<_>>()
					     .join("\n----------------\n")
				);
			} else {
				posts.iter().for_each(|p| println!("{}", p.id));
			}
			
			Ok(())
		},
		Err(e) => {
			match e {
				Error::MaxLimit(max) => {
					Err(format!(
						"{} is above the max limit for ordered queries ({}).",
						max,
						limit
					))
				},
				Error::Http(code) => {
					Err(format!("HTTP error: {}", code))
				},
				Error::Serial(msg) => {
					Err(format!("Serialization error: {}", msg))
				},
				Error::Redirect(msg) => {
					Err(format!("Redirect error: {}", msg))
				},
				Error::CannotSendRequest(msg) => {
					Err(format!("Couldn't send request: {}", msg))
				},
				Error::CannotCreateClient(msg) => {
					Err(format!("Couldn't create client: {}", msg))
				},
			}
		},
	}
}

fn main() {
	// CLI Arguments parsing
	let matches =
		App::new("get621")
			.version("1.1.0_dev")
			.author("nasso <nassomails ~ at ~ gmail {dot} com>")
			.about("E621/926 command line tool")
			.arg(Arg::with_name("children")
				.short("c")
				.long("children")
				.help("Take the children of search results"))
			.arg(Arg::with_name("json")
				.short("j")
				.long("json")
				.help("Output the results as JSON on the standard ouptut"))
			.arg(Arg::with_name("limit")
				.short("l")
				.long("limit")
				.default_value("1")
				.takes_value(true)
				.validator(|v| valid_parse::<u64>(&v, "Must be a positive integer."))
				.help("Maximum search result count"))
			.arg(Arg::with_name("output")
				.short("o")
				.long("output")
				.help("Download and output posts to stdout (unseparated)"))
			.arg(Arg::with_name("parents")
				.short("p")
				.long("parents")
				.help("Take the parent post of each search result, if any"))
			.arg(Arg::with_name("pool")
				.short("P")
				.long("pool")
				.takes_value(true)
				.help("Search only in the posts from the pool (ordered)"))
			.arg(Arg::with_name("save")
				.short("s")
				.long("save")
				.help("Download every result to ./<post_id>.<ext>"))
			.arg(Arg::with_name("verbose")
				.short("v")
				.long("verbose")
				.conflicts_with("output")
				.help("Enable verbose output to standard output"))
			.arg(Arg::with_name("tags")
				.raw(true)
				.help("Search tags"))
		.get_matches();
	
	let verbose = matches.is_present("verbose");
	
	::std::process::exit(match run_app(matches) {
		Ok(_) => 0,
		Err(msg) => {
			if verbose {
				eprintln!("{}", msg);
			}
			
			1
		}
	})
}
