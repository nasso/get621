use crate::common::{self, output_mode_check, output_posts, save_posts, valid_parse};
use clap::{Arg, ArgMatches};
use get621::Get621;

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
        Arg::with_name("pool_id")
            .short("P")
            .long("pool")
            .takes_value(true)
            .validator(|v| valid_parse::<u64>(&v, "Must be a positive integer."))
            .help("Search for posts in the given pool ID (ordered)"),
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
            .help("Set output mode; one of: id, json, raw, verbose, none"),
        Arg::with_name("tags")
            .index(1)
            .multiple(true)
            .allow_hyphen_values(true)
            .conflicts_with("pool_id")
            .help("Search tags"),
    ]
}

// get621 ...
pub fn run(matches: &ArgMatches) -> common::Result<()> {
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
        let tags = matches
            .values_of("tags")
            .map_or_else(|| Vec::new(), |v| v.collect::<Vec<_>>());
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
    output_posts(&g6, &posts, matches.value_of("output_mode").unwrap())?;

    if matches.is_present("save") {
        save_posts(&g6, &posts, pool_id)?;
    }

    Ok(())
}
