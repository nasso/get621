extern crate clap;

use clap::{Arg, App};

use get621::{self, Error};

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
				.help("Download results to ./<post_id>.<ext>"))
			.arg(Arg::with_name("verbose")
				.short("v")
				.long("verbose")
				.conflicts_with("output")
				.help("Enable verbose output to standard output"))
			.arg(Arg::with_name("tags")
				.raw(true)
				.help("Search tags"))
		.get_matches();
	
	let limit = match matches.value_of("limit").unwrap().parse() {
		Ok(v) => v,
		Err(_) => {
			eprintln!("The limit must be a positive integer.");
			1
		},
	};
	
	// Get posts
	if let Some(tags) = matches.values_of("tags") {
		match get621::list(&tags.collect::<Vec<_>>(), limit) {
			Ok(posts) => {
				
			},
			Err(e) => {
				if matches.is_present("verbose") {
					match e {
						Error::MaxLimit(max) => {
							eprintln!(
								"{} is above the max limit for ordered queries ({}).",
								max,
								limit
							)
						},
						_ => eprintln!("Something happened."),
					}
				}	
			},
		}
	} else {
		println!("No tags!");
	}
}
