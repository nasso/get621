use crate::common::{self, output_mode_check, output_posts, post_map, save_post, valid_parse};
use clap::{crate_version, Arg, ArgMatches};
use futures::{pin_mut, stream, StreamExt};
use rs621::client::Client;

pub fn args<'a, 'b>() -> Vec<Arg<'a, 'b>> {
    vec![
        Arg::with_name("url")
            .short("u")
            .long("url")
            .default_value("https://e926.net")
            .help("The URL of the server where requests should be made"),
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
            .help("Set output mode; one of: id, raw, verbose"),
        Arg::with_name("tags")
            .index(1)
            .multiple(true)
            .allow_hyphen_values(true)
            .help("Search tags"),
    ]
}

// get621 ...
pub async fn run(matches: &ArgMatches<'_>) -> common::Result<()> {
    let limit: u64 = matches.value_of("limit").unwrap().parse().unwrap();
    let flag_save = matches.is_present("save");

    // Create client
    let client = Client::new(
        matches.value_of("url").unwrap(),
        &format!("get621/{} (by nasso on e621)", crate_version!()),
    )?;

    // search tags
    let tags = matches
        .values_of("tags")
        .map_or_else(|| Vec::new(), |v| v.collect::<Vec<_>>());

    // Request
    let post_stream = client.post_search(&tags[..]).take(limit as usize);

    // Get the posts
    let posts = post_map(&client, matches.into(), post_stream).await?;
    let post_stream = stream::iter(posts).then(|post| async move {
        if flag_save {
            if let Err(e) = save_post(&post, None).await {
                eprintln!("Error when saving #{}: {}", post.id, e);
            }
        }

        post
    });
    pin_mut!(post_stream);

    // Do whatever the user asked us to do
    let output_mode = matches.value_of("output_mode").unwrap().into();

    output_posts(post_stream, output_mode).await?;

    Ok(())
}
