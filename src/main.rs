extern crate clap;

use std::io;
use std::fs::File;
use std::str::FromStr;

use clap::{Arg, App, ArgMatches};

use get621::Get621;

enum Error {
	Get621Error(get621::Error),
	IOError(io::Error),
}

impl From<get621::Error> for Error {
	fn from(e: get621::Error) -> Self {
		Error::Get621Error(e)
	}
}

impl From<io::Error> for Error {
	fn from(e: io::Error) -> Self {
		Error::IOError(e)
	}
}

fn valid_parse<T: FromStr>(v: &str, emsg: &str) -> Result<(), String> {
	match v.parse::<T>() {
		Ok(_) => Ok(()),
		Err(_) => Err(String::from(emsg))
	}
}

fn translate_error(e: Error) -> String {
	match e {
		Error::Get621Error(e) => match e {
			get621::Error::AboveLimit(limit, max) => {
				format!("{} is above the max limit for ordered queries ({})", limit, max)
			},
			get621::Error::Http(code) => format!("HTTP error: {}", code),
			get621::Error::Serial(msg) => format!("Serialization error: {}", msg),
			get621::Error::Redirect(msg) => format!("Redirect error: {}", msg),
			get621::Error::CannotSendRequest(msg) => format!("Couldn't send request: {}", msg),
			get621::Error::CannotCreateClient(msg) => format!("Couldn't create client: {}", msg),
			get621::Error::Download(msg) => format!("Error when downloading the post: {}", msg),
		},
		
		Error::IOError(e) => match e.kind() {
			io::ErrorKind::NotFound => {
				format!("One of the directory components of the file path does not exist.")
			},
			io::ErrorKind::PermissionDenied => format!("File access permission denied."),
			_ => format!("IO Error: {:?}", e),
		}
	}
}

fn run_app(matches: ArgMatches) -> Result<(), Error> {
	// Post result list
	let mut posts = Vec::new();
	let mut pool_id = None;
	
	// Create client
	let g6 = Get621::init()?;
	
	// Request
	let mut res = if matches.is_present("pool_id") {
		let id = matches.value_of("pool_id").unwrap().parse().unwrap();
		pool_id = Some(id);
		
		g6.pool(id)?
	} else {
		let tags = matches.values_of("tags").map_or_else(|| Vec::new(), |v| v.collect::<Vec<_>>());
		let limit = matches.value_of("limit").unwrap().parse().unwrap();
		
		g6.list(&tags, limit)?
	};
	
	// Get the posts
	if matches.is_present("parents") {
		while !res.is_empty() {
			let p = res.pop().unwrap();
			
			if let Some(id) = p.parent_id {
				posts.push(g6.get_post(id)?);
			}
		}
	} else if matches.is_present("children") {
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
	
	// Do whatever the user asked us to do
	if matches.is_present("verbose") {
		println!(
			"{}",
			posts.iter().map(|p| p.to_string())
			     .collect::<Vec<_>>()
			     .join("\n----------------\n")
		);
	} else if matches.is_present("json") {
		println!(
			"[{}]",
			posts.iter().map(|p| p.raw.clone())
			     .collect::<Vec<_>>()
			     .join(",")
		);
	} else if matches.is_present("output") {
		let mut stdout = io::stdout();
		
		for p in posts.iter().filter(|p| !p.status.is_deleted()) {
			g6.download(p, &mut stdout)?;
		}
	} else {
		posts.iter().for_each(|p| println!("{}", p.id));
	}
	
	if matches.is_present("save") {
		for (i, p) in posts.iter().filter(|p| !p.status.is_deleted()).enumerate() {
			let mut file = if let Some(id) = pool_id {
				File::create(format!("{}-{}_{}.{}", id, i + 1, p.id, p.file_ext.as_ref().unwrap()))?
			} else {
				File::create(format!("{}.{}", p.id, p.file_ext.as_ref().unwrap()))?
			};
			
			g6.download(p, &mut file)?;
		}
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
			.arg(Arg::with_name("pool_id")
				.short("P")
				.long("pool")
				.takes_value(true)
				.validator(|v| valid_parse::<u64>(&v, "Must be a positive integer."))
				.help("Search for posts in the given pool ID (ordered)"))
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
				.conflicts_with("pool_id")
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
