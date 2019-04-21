use crate::common::{
    self, expand_paths, output_mode_check, output_posts, save_posts, valid_parse, OutputMode,
};
use clap::{Arg, ArgMatches};
use get621::Get621;
use regex::Regex;
use reqwest::{self, multipart};
use scraper::{Html, Selector};
use std::{fs::File, io, path::Path};
use tempfile::tempfile;

// arguments of the subcommand
pub fn args<'a, 'b>() -> Vec<Arg<'a, 'b>> {
    vec![
        Arg::with_name("source")
            .index(1)
            .required(true)
            .multiple(true)
            .allow_hyphen_values(true)
            .required(true)
            .help("Files or folders to reverse search; can be a glob pattern"),
        Arg::with_name("similarity")
            .short("S")
            .long("similarity")
            .takes_value(true)
            .default_value("90")
            .validator(|v| valid_parse::<f64>(&v, "Must be a floating point value."))
            .help("Set the similarity threshold for matching posts (in percents)"),
        Arg::with_name("save")
            .short("s")
            .long("save")
            .help("Download all matching posts to ./<post_id>.<ext>"),
        Arg::with_name("direct")
        Arg::with_name("direct_save")
            .short("d")
            .long("direct-save")
            .overrides_with("save")
            .conflicts_with("output_mode")
            .help("Download posts directly without requesting other post information (faster)"),
        Arg::with_name("output_mode")
            .short("o")
            .long("output")
            .takes_value(true)
            .default_value("verbose")
            .validator(output_mode_check)
            .help("Set output mode; one of: id, json, raw, verbose"),
    ]
}

struct ReverseSearchResult {
    id: u64,
    md5: String,
}

static DIRECT_FORMATS: [&str; 3] = ["png", "jpg", "gif"];

// macro for verbose output -> println!
macro_rules! verbose_println {
    ($is_verbose:expr) => ({
        if $is_verbose {
            println!()
        }
    });
    ($is_verbose:expr, $($arg:tt)*) => ({
        if $is_verbose {
            println!($($arg)*)
        }
    });
}

// macro for verbose output -> print!
macro_rules! verbose_print {
    ($is_verbose:expr) => ({
        if $is_verbose {
            print!()
        }
    });
    ($is_verbose:expr, $($arg:tt)*) => ({
        if $is_verbose {
            print!($($arg)*)
        }
    });
}

// do the reverse search and return the results
fn reverse_search(path: &Path, min_similarity: f64) -> common::Result<Vec<ReverseSearchResult>> {
    lazy_static! {
        static ref RESULT_DIVS_SELECTOR: Selector = Selector::parse(
            "#pages > div:not(:first-child):not(#show1):not(#more1), #more1 > .pages > div"
        )
        .unwrap();
        static ref E6_LINK_REGEX: Regex =
            Regex::new(r#"https://e621\.net/post/show/(\d+)"#).unwrap();
        static ref E6_MD5_REGEX: Regex =
            Regex::new(r#"e621/[0-9a-f]{2}/[0-9a-f]{2}/([0-9a-f]{32})\.jpg"#).unwrap();
        static ref POST_SIMILARITY_REGEX: Regex = Regex::new(r#"(\d+)% similarity"#).unwrap();
    }

    let form = multipart::Form::new()
        .text("service[]", "0")
        .text("MAX_FILE_SIZE", "8388608")
        .file("file", path)?
        .text("url", "");

    let mut resp = common::CLIENT
        .post("http://iqdb.harry.lu/")
        .multipart(form)
        .send()?;

    let doc = Html::parse_document(&resp.text()?);

    Ok(doc
        // for each div corresponding to a result
        .select(&RESULT_DIVS_SELECTOR)
        // get the inner html
        .map(|result| result.inner_html())
        // filter out posts below threshold
        .filter(|html| {
            match POST_SIMILARITY_REGEX
                .captures(&html)
                .and_then(|caps| caps.get(1))
                .and_then(|cap| cap.as_str().parse::<f64>().ok())
            {
                Some(val) => val >= min_similarity,
                None => false,
            }
        })
        // get the e621 link
        .map(|html| {
            (
                E6_LINK_REGEX
                    .captures(&html)
                    .and_then(|caps| caps.get(1))
                    .and_then(|cap| cap.as_str().parse::<u64>().ok()),
                E6_MD5_REGEX
                    .captures(&html)
                    .and_then(|caps| caps.get(1))
                    .map(|cap| cap.as_str().to_string()),
            )
        })
        // retrieve posts
        .filter(|(id, md5)| id.is_some() && md5.is_some())
        .map(|(id, md5)| ReverseSearchResult {
            id: id.unwrap(),
            md5: md5.unwrap(),
        })
        .collect())
}

// get621 reverse ...
pub fn run(matches: &ArgMatches) -> common::Result<()> {
    let arg_source = matches.values_of("source").unwrap().collect::<Vec<_>>();
    let arg_similarity = matches.value_of("similarity").unwrap().parse().unwrap();
    let arg_outputmode = matches.value_of("output_mode").unwrap();
    let flag_direct = matches.is_present("direct_save");
    let flag_save = matches.is_present("save");

    let vb = match arg_outputmode.into() {
        OutputMode::Verbose => true,
        _ => false,
    };

    // Create client
    let g6 = if flag_direct {
        None
    } else {
        Some(Get621::init()?)
    };

    expand_paths(&arg_source)?
        .into_iter()
        // only take files!
        .filter(|path| path.is_file())
        // verbose output
        .inspect(|path| {
            verbose_println!(vb, "Looking for {}", path.display());
            verbose_println!(vb, "================================");
        })
        // do the reverse search
        .filter_map(|path| {
            reverse_search(&path, arg_similarity)
                .or_else(|e| {
                    eprintln!("Error: {}", e);
                    Err(())
                })
                .ok()
        })
        // process the results
        .map(|results| {
            if flag_direct {
                // don't even ask e621 for anything
                for result in results.into_iter() {
                    verbose_println!(vb, "Found MD5: {}", result.md5);

                    let mut temp_dl_dest = tempfile()?;

                    // test for each format as it could be any
                    for ext in DIRECT_FORMATS.iter() {
                        verbose_print!(vb, "Looking for {}...", ext);

                        if common::download(
                            &format!(
                                "https://static1.e621.net/data/{}/{}/{}.{}",
                                &result.md5[0..2],
                                &result.md5[2..4],
                                &result.md5,
                                ext
                            ),
                            &mut temp_dl_dest,
                        )
                        .is_err()
                        {
                            verbose_println!(vb, " not found");
                        } else {
                            // copy the downloaded data
                            let mut final_dest = File::create(format!("{}.{}", result.id, ext))?;

                            io::copy(&mut temp_dl_dest, &mut final_dest)?;

                            verbose_println!(vb, " downloaded to {}.{}", result.id, ext);

                            break;
                        }
                    }
                }
            } else {
                // regular post fetching
                let posts = results
                    .into_iter()
                    .filter_map(|result| match g6 {
                        Some(ref g6) => g6.get_post(result.id).ok(),
                        _ => None,
                    })
                    .collect();

                // output all the posts as usual
                output_posts(&posts, arg_outputmode)?;

                // maybe save them
                if flag_save {
                    save_posts(&posts, None)?;
                }
            }

            // verbose empty line
            verbose_println!(vb);

            Ok(())
        })
        // print all the errors
        .for_each(|r: common::Result<()>| match r {
            Err(e) => eprintln!("Error: {}", e),
            _ => (),
        });

    Ok(())
}
