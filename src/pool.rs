use crate::common::{
    self, output_mode_check, output_posts, post_map, save_post, valid_parse, Error,
};
use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use futures::{pin_mut, stream, StreamExt};
use rs621::{client::Client, pool::PoolSearch};

pub fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("pool")
        .about("Pool related commands")
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
                .help("Set output mode; one of: id, raw, verbose"),
        )
        .arg(
            Arg::with_name("id")
                .index(1)
                .required(true)
                .validator(|v| valid_parse::<u64>(&v, "Must be a positive integer."))
                .help("The ID of the pool"),
        )
}

pub async fn run(url: &str, matches: &ArgMatches<'_>) -> common::Result<()> {
    let pool_id: u64 = matches.value_of("id").unwrap().parse().unwrap();
    let flag_save = matches.is_present("save");

    // Create client
    let client = Client::new(
        url,
        &format!("get621/{} (by nasso on e621)", crate_version!()),
    )?;

    // Get the posts
    let post_ids = client
        .pool_search(PoolSearch::new().id(vec![pool_id]))
        .next()
        .await
        .ok_or(Error::PoolNotFound)??
        .post_ids;
    let posts = client.get_posts(&post_ids);

    let posts = post_map(&client, matches.into(), posts).await?;
    let post_stream = stream::iter(posts)
        .enumerate()
        .then(|(i, post)| async move {
            if flag_save {
                if let Err(e) = save_post(&post, &format!("{}-{}_", pool_id, i)[..]).await {
                    eprintln!("Error when saving #{}: {}", post.id, e);
                }
            }

            post
        });
    pin_mut!(post_stream);

    // Do whatever the user asked us to do
    output_posts(post_stream, matches.value_of("output_mode").unwrap().into()).await?;

    Ok(())
}
