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

fn translate_error(e: Error) -> String {
	match e {
		Error::AboveLimit(limit, max) => {
			format!("{} is above the max limit for ordered queries ({})", limit, max)
		},
		Error::Http(code) => format!("HTTP error: {}", code),
		Error::Serial(msg) => format!("Serialization error: {}", msg),
		Error::Redirect(msg) => format!("Redirect error: {}", msg),
		Error::CannotSendRequest(msg) => format!("Couldn't send request: {}", msg),
		Error::CannotCreateClient(msg) => format!("Couldn't create client: {}", msg),
	}
}

fn run_app(matches: ArgMatches) -> get621::Result<()> {
	// Get args
	let limit = matches.value_of("limit").unwrap().parse().unwrap();
	let verbose = matches.is_present("verbose");
	let json = matches.is_present("json");
	let tags = matches.values_of("tags").map_or_else(|| Vec::new(), |v| v.collect::<Vec<_>>());
	
	let parents = matches.is_present("parents");
	let children = matches.is_present("children");
	
	let g6 = Get621::init()?;
	let mut res = g6.list(&tags, limit)?;
	
	let mut posts = Vec::new();
	
	if parents {
		while !res.is_empty() {
			let p = res.pop().unwrap();
			
			if let Some(id) = p.parent_id {
				posts.push(g6.get_post(id)?);
			}
		}
	} else if children {
		while !res.is_empty() {
			let p = res.pop().unwrap();
			
			if let Some(c) = p.children {
				for id in c.iter() {
					posts.push(g6.get_post(*id)?);
				}
			}
		}
	} else {
		posts.append(&mut res);
	}
	
	// Get posts
	if verbose {
		println!(
			"{}",
			posts.iter().map(|p| p.to_string())
			     .collect::<Vec<_>>()
			     .join("\n----------------\n")
		);
	} else if json {
		println!(
			"[{}]",
			posts.iter().map(|p| p.raw.clone())
			     .collect::<Vec<_>>()
			     .join(",")
		);
	} else {
		posts.iter().for_each(|p| println!("{}", p.id));
	}
	
	Ok(())
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
				.conflicts_with("parents")
				.help("Take the children of search results"))
			.arg(Arg::with_name("json")
				.short("j")
				.long("json")
				.conflicts_with("verbose")
				.conflicts_with("output")
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
				.conflicts_with("verbose")
				.conflicts_with("json")
				.help("Download and output posts to stdout (unseparated)"))
			.arg(Arg::with_name("parents")
				.short("p")
				.long("parents")
				.conflicts_with("children")
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
				.conflicts_with("json")
				.help("Enable verbose output to standard output"))
			.arg(Arg::with_name("tags")
				.raw(true)
				.help("Search tags"))
		.get_matches();
	
	::std::process::exit(match run_app(matches) {
		Ok(_) => 0,
		Err(e) => {
			eprintln!("{}", translate_error(e));
			1
		}
	})
}
