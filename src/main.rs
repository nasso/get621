extern crate clap;

use clap::{Arg, App};

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
				.help("Enable verbose output to standard output"))
			.arg(Arg::with_name("tags")
				.raw(true)
				.help("Search tags"))
		.get_matches();
	
	// Get posts
	
}
