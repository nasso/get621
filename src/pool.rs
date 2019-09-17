use crate::common::{self, output_mode_check, output_posts, post_map, save_posts, valid_parse};
use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use rs621::client::Client;

pub fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("pool")
        .about("E621/926 pool related utils")
        .arg(
            Arg::with_name("children")
                .short("c")
                .long("children")
                .conflicts_with("parents")
                .help("Take the children of search results"),
        )
        .arg(
            Arg::with_name("parents")
                .short("p")
                .long("parents")
                .conflicts_with("children")
                .help("Take the parent post of each search result, if any"),
        )
        .arg(
            Arg::with_name("save")
                .short("s")
                .long("save")
                .help("Download every result to ./<pool_id>-i_<post_id>.<ext>"),
        )
        .arg(
            Arg::with_name("output_mode")
                .short("o")
                .long("output")
                .takes_value(true)
                .default_value("verbose")
                .validator(output_mode_check)
                .help("Set output mode; one of: id, json, raw, verbose"),
        )
        .arg(
            Arg::with_name("id")
                .index(1)
                .required(true)
                .validator(|v| valid_parse::<u64>(&v, "Must be a positive integer."))
                .help("The ID of the pool"),
        )
}

pub fn run(matches: &ArgMatches) -> common::Result<()> {
    let id: u64 = matches.value_of("id").unwrap().parse().unwrap();

    // Create client
    let client = Client::new(&format!("get621/{} (by nasso on e621)", crate_version!()))?;

    // Get the posts
    let posts = post_map(
        &client,
        matches.into(),
        client.get_pool(id)?.posts.into_iter().map(Result::Ok),
    )?;

    // Do whatever the user asked us to do
    output_posts(&posts, matches.value_of("output_mode").unwrap().into())?;

    if matches.is_present("save") {
        save_posts(&posts, None)?;
    }

    Ok(())
}
