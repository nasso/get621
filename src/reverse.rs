use crate::common::{
    self, download, expand_paths, output_mode_check, output_posts, save_post, valid_parse, Error,
    OutputMode, Result,
};
use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use futures::{pin_mut, StreamExt};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{
    self,
    multipart::{self, Part},
};
use rs621::client::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::{fs::File, io::Read, path::Path};

// arguments of the subcommand
pub fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("reverse")
        .about("Similar image search (experimental)")
        .arg(
            Arg::with_name("source")
                .index(1)
                .required(true)
                .multiple(true)
                .allow_hyphen_values(true)
                .required(true)
                .help("Files or folders to reverse search; can be a glob pattern"),
        )
        .arg(
            Arg::with_name("similarity")
                .short("S")
                .long("similarity")
                .takes_value(true)
                .default_value("90")
                .validator(|v| valid_parse::<f64>(&v, "Must be a floating point value."))
                .help("Set the similarity threshold for matching posts (in percents)"),
        )
        .arg(
            Arg::with_name("save")
                .short("s")
                .long("save")
                .help("Download all matching posts to ./<post_id>.<ext>"),
        )
        .arg(
            Arg::with_name("direct_save")
                .short("d")
                .long("direct-save")
                .overrides_with("save")
                .conflicts_with("output_mode")
                .help("Download posts directly without requesting other post information (faster)"),
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
}

#[derive(Deserialize)]
struct ReverseSearchResult {
    id: u64,
    file_ext: String,
    file_url: String,
}

async fn get_csrf_token(page_url: &str) -> Result<(String, String)> {
    lazy_static! {
        static ref SELECT_META: Selector = Selector::parse("meta[name=\"csrf-token\"]").unwrap();
    }

    let response = common::CLIENT
        .get(page_url)
        .header(
            "User-Agent",
            &format!("get621/{} (by nasso on e621)", crate_version!()),
        )
        .send()
        .await?;

    let cookie = response
        .headers()
        .get("set-cookie")
        .ok_or(Error::AuthTokenNotFound)?
        .to_str()
        .or(Err(Error::AuthTokenNotFound))?
        .into();

    let doc = Html::parse_document(&response.text().await?);

    Ok((
        doc.select(&SELECT_META)
            .next()
            .and_then(|tag| tag.value().attr("content"))
            .ok_or(Error::AuthTokenNotFound)?
            .into(),
        cookie,
    ))
}

// do the reverse search and return the results
async fn reverse_search(
    url: &str,
    path: &Path,
    min_similarity: f64,
) -> Result<Vec<ReverseSearchResult>> {
    lazy_static! {
        static ref SELECT_RESULTS: Selector = Selector::parse(".post-preview").unwrap();
        static ref POST_SIMILARITY_REGEX: Regex = Regex::new(r#"Similarity:?\s*(\d+)"#).unwrap();
    }

    let (token, cookie) = get_csrf_token(&format!("{}/iqdb_queries", url)).await?;

    let form = multipart::Form::new()
        .text("authenticity_token", token)
        .text("url", "")
        .part("file", {
            let file_name = path
                .file_name()
                .map(|filename| filename.to_string_lossy().into_owned());
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            let mut file = File::open(path)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;

            let field = Part::bytes(bytes).mime_str(mime.essence_str())?;

            if let Some(file_name) = file_name {
                field.file_name(file_name)
            } else {
                field
            }
        });

    let mut json: serde_json::Value = common::CLIENT
        .post(format!("{}/iqdb_queries.json", url))
        .header(
            "User-Agent",
            &format!("get621/{} (by nasso on e621)", crate_version!()),
        )
        .header("Cookie", cookie)
        .multipart(form)
        .send()
        .await?
        .json()
        .await?;

    let mut results = Vec::new();

    for candidate in json
        .as_array_mut()
        .ok_or(Error::IqdbQueryError)?
        .into_iter()
    {
        if let Some(similarity) = candidate["score"].as_f64() {
            if similarity >= min_similarity {
                results.push(serde_json::from_value(candidate["post"]["posts"].take())?);
            }
        } else {
            return Err(Error::IqdbQueryError);
        }
    }

    Ok(results)
    /*
    let doc = Html::parse_document(&doc);

    Ok(doc
        // for each result
        .select(&SELECT_RESULTS)
        // filter out posts below threshold
        .filter(|elem| {
            match POST_SIMILARITY_REGEX
                .captures(&elem.inner_html())
                .and_then(|caps| caps.get(1))
                .and_then(|cap| cap.as_str().parse::<f64>().ok())
            {
                Some(val) => val >= min_similarity,
                None => false,
            }
        })
        // get the e621 link
        .map(|elem| {
            (
                elem.value().attr("data-id"),
                elem.value().attr("data-file-ext"),
                elem.value().attr("data-file-url"),
            )
        })
        // retrieve posts
        .filter(|(id, _, _)| id.is_some())
        .map(|(id, ext, url)| ReverseSearchResult {
            id: id.unwrap().parse::<u64>().unwrap(),
            file_ext: ext.unwrap().into(),
            file_url: url.map(String::from),
        })
        .collect())
    */
}

// get621 reverse ...
pub async fn run(url: &str, matches: &ArgMatches<'_>) -> Result<()> {
    let arg_source = matches.values_of("source").unwrap().collect::<Vec<_>>();
    let arg_similarity = matches.value_of("similarity").unwrap().parse().unwrap();
    let arg_outputmode = matches.value_of("output_mode").unwrap();
    let flag_save = matches.is_present("save");

    let vb = match arg_outputmode.into() {
        OutputMode::Verbose => true,
        _ => false,
    };

    // Create client
    let client = if matches.is_present("direct_save") {
        None
    } else {
        Some(Client::new(
            url,
            &format!("get621/{} (by nasso on e621)", crate_version!()),
        )?)
    };

    // macro for verbose output -> println!
    macro_rules! verbose_println {
        ($($arg:tt)*) => { if vb { println!($($arg)*) } }
    }

    let file_paths = expand_paths(&arg_source)?
        .into_iter()
        .filter(|path| path.is_file());

    for path in file_paths {
        verbose_println!("Looking for {}", path.display());
        verbose_println!("================================");

        // do the reverse search
        let results = reverse_search(url, &path, arg_similarity).await?;

        if results.is_empty() {
            verbose_println!("No result.");
        } else if let Some(ref client) = client {
            // just get post information
            let post_ids = results.into_iter().map(|r| r.id).collect::<Vec<_>>();
            let posts = client
                .get_posts(&post_ids)
                .map(|r| r.map_err(Error::from))
                .filter_map(|res| async move {
                    match res {
                        Ok(post) => Some(post),
                        Err(e) => {
                            eprintln!("{}", e);
                            None
                        }
                    }
                })
                .then(|post| async move {
                    if flag_save {
                        if let Err(e) = save_post(&post, None).await {
                            eprintln!("Error when saving #{}: {}", post.id, e);
                        }
                    }

                    post
                });
            pin_mut!(posts);

            // output all the posts as usual
            output_posts(posts, arg_outputmode.into()).await?;
        } else {
            // no client = directly download the image
            for result in results.into_iter() {
                verbose_println!("Downloading {}...", result.file_url);

                let dest_path = format!("{}.{}", result.id, result.file_ext);
                let mut dest = File::create(&dest_path)?;

                download(result.file_url, &mut dest).await?;
            }
        }

        // verbose empty line
        verbose_println!();
    }

    Ok(())
}
