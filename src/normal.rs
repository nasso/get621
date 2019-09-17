use crate::common::{self, output_mode_check, output_posts, post_map, save_posts, valid_parse};
use clap::{crate_version, Arg, ArgMatches};
use rs621::client::Client;

pub fn args<'a, 'b>() -> Vec<Arg<'a, 'b>> {
    vec![
        Arg::with_name("children")
            .short("c")
            .long("children")
            .conflicts_with("parents")
            .help("Take the children of search results"),
        Arg::with_name("limit")
            .short("l")
            .long("limit")
            .default_value("1")
            .takes_value(true)
            .validator(|v| valid_parse::<u64>(&v, "Must be a positive integer."))
            .help("Maximum search result count"),
        Arg::with_name("parents")
            .short("p")
            .long("parents")
            .conflicts_with("children")
            .help("Take the parent post of each search result, if any"),
        Arg::with_name("save")
            .short("s")
            .long("save")
            .help("Download every result to ./<post_id>.<ext>"),
        Arg::with_name("output_mode")
            .short("o")
            .long("output")
            .takes_value(true)
            .default_value("verbose")
            .validator(output_mode_check)
            .help("Set output mode; one of: id, json, raw, verbose"),
        Arg::with_name("tags")
            .index(1)
            .multiple(true)
            .allow_hyphen_values(true)
            .help("Search tags"),
    ]
}

// get621 ...
pub fn run(matches: &ArgMatches) -> common::Result<()> {
    let limit: u64 = matches.value_of("limit").unwrap().parse().unwrap();

    // Create client
    let client = Client::new(&format!("get621/{} (by nasso on e621)", crate_version!()))?;

    // search tags
    let tags = matches
        .values_of("tags")
        .map_or_else(|| Vec::new(), |v| v.collect::<Vec<_>>());

    // Request
    let post_iter = client.post_search(&tags[..]).take(limit as usize);

    // Get the posts
    let posts = post_map(&client, matches.into(), post_iter)?;

    // Do whatever the user asked us to do
    output_posts(&posts, matches.value_of("output_mode").unwrap().into())?;

    if matches.is_present("save") {
        save_posts(&posts, None)?;
    }

    Ok(())
}
