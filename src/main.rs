#[macro_use]
extern crate lazy_static;

use clap::{App, Arg, ArgMatches, SubCommand};
use get621::{Get621, Post};
use globwalk;
use regex::Regex;
use reqwest::{self, multipart};
use scraper::{Html, Selector};
use std::{
    fmt,
    fs::File,
    io::{self, Write},
    str::FromStr,
};

fn valid_parse<T: FromStr>(v: &str, emsg: &str) -> Result<(), String> {
    match v.parse::<T>() {
        Ok(_) => Ok(()),
        Err(_) => Err(emsg.to_string()),
    }
}

enum Error {
    Get621Error(get621::Error),
    IOError(io::Error),
    GlobError(globwalk::GlobError),
    ReqwestError(reqwest::Error),
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

impl From<globwalk::GlobError> for Error {
    fn from(e: globwalk::GlobError) -> Self {
        Error::GlobError(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::ReqwestError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Get621Error(e) => write!(f, "{}", e),
            Error::IOError(e) => write!(f, "{}", e),
            Error::GlobError(e) => write!(f, "{}", e),
            Error::ReqwestError(e) => write!(f, "{}", e),
        }
    }
}

enum OutputMode {
    Id,
    Json,
    Raw,
    Verbose,
    None,
}

impl From<&str> for OutputMode {
    fn from(s: &str) -> Self {
        match s {
            "id" => OutputMode::Id,
            "json" => OutputMode::Json,
            "raw" => OutputMode::Raw,
            "verbose" => OutputMode::Verbose,
            _ => OutputMode::None,
        }
    }
}

fn output_mode_check(v: String) -> Result<(), String> {
    if v == "id" || v == "json" || v == "raw" || v == "verbose" || v == "none" {
        Ok(())
    } else {
        Err("Must be one of: id, json, raw, verbose, none".to_string())
    }
}

fn output_posts<T: Into<OutputMode>>(g6: &Get621, posts: &Vec<Post>, mode: T) -> Result<(), Error> {
    match mode.into() {
        OutputMode::Id => {
            posts.iter().for_each(|p| println!("{}", p.id));

            Ok(())
        }

        OutputMode::Json => {
            println!(
                "[{}]",
                posts
                    .iter()
                    .map(|p| p.raw.clone())
                    .collect::<Vec<_>>()
                    .join(",")
            );

            Ok(())
        }

        OutputMode::Raw => {
            let mut stdout = io::stdout();

            for p in posts.iter().filter(|p| !p.status.is_deleted()) {
                g6.download(p, &mut stdout)?;
            }

            Ok(())
        }

        OutputMode::Verbose => {
            println!(
                "{}",
                posts
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join("\n----------------\n")
            );

            Ok(())
        }

        _ => Ok(()),
    }
}

fn save_posts(g6: &Get621, posts: &Vec<Post>, pool_id: Option<u64>) -> Result<(), Error> {
    for (i, p) in posts.iter().filter(|p| !p.status.is_deleted()).enumerate() {
        let mut file = if let Some(id) = pool_id {
            File::create(format!(
                "{}-{}_{}.{}",
                id,
                i + 1,
                p.id,
                p.file_ext.as_ref().unwrap()
            ))?
        } else {
            File::create(format!("{}.{}", p.id, p.file_ext.as_ref().unwrap()))?
        };

        g6.download(p, &mut file)?;
    }

    Ok(())
}

// get621 ...
fn run_normal(matches: &ArgMatches) -> Result<(), Error> {
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

// get621 reverse ...
fn run_reverse(matches: &ArgMatches) -> Result<(), Error> {
    lazy_static! {
        static ref CLIENT: reqwest::Client = reqwest::Client::new();
        static ref RESULT_DIVS_SELECTOR: Selector = Selector::parse(
            "#pages > div:not(:first-child):not(#show1):not(#more1), #more1 > .pages > div"
        )
        .unwrap();
        static ref E6_LINK_REGEX: Regex =
            Regex::new(r#"https://e621\.net/post/show/(\d+)"#).unwrap();
        static ref POST_SIMILARITY_REGEX: Regex = Regex::new(r#"(\d+)% similarity"#).unwrap();
    }

    // Create client
    let g6 = Get621::init()?;

    let arg_source = matches.value_of("source").unwrap();
    let arg_similarity = matches.value_of("similarity").unwrap().parse().unwrap();
    let arg_outputmode = matches.value_of("output_mode").unwrap();

    let is_verbose = match arg_outputmode.into() {
        OutputMode::Verbose => true,
        _ => false,
    };

    for entry in globwalk::glob(arg_source)?
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
    {
        let path = entry.path();

        if is_verbose {
            println!("Looking for {}", path.to_string_lossy());
            println!("================================");
        }

        let form = multipart::Form::new()
            .text("service[]", "0")
            .text("MAX_FILE_SIZE", "8388608")
            .file("file", path)?
            .text("url", "");

        let mut resp = CLIENT
            .post("http://iqdb.harry.lu/")
            .multipart(form)
            .send()?;

        let doc = Html::parse_document(&resp.text()?);

        let posts = doc
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
                    Some(val) => val >= arg_similarity,
                    None => false,
                }
            })
            // get the e621 link
            .filter_map(|html| {
                E6_LINK_REGEX
                    .captures(&html)
                    .and_then(|caps| caps.get(1))
                    .and_then(|cap| cap.as_str().parse::<u64>().ok())
            })
            // retrieve posts
            .filter_map(|id| g6.get_post(id).ok())
            .collect();

        output_posts(&g6, &posts, arg_outputmode)?;

        if matches.is_present("save") {
            if is_verbose {
                print!("Downloading...");
                io::stdout().flush()?;
            }

            save_posts(&g6, &posts, None)?;

            if is_verbose {
                println!("\rDownload complete.");
            }
        }

        if is_verbose {
            println!();
        }
    }

    Ok(())
}

// runs the program
fn run_app(matches: &ArgMatches) -> Result<(), Error> {
    if let Some(matches) = matches.subcommand_matches("reverse") {
        run_reverse(matches)
    } else {
        run_normal(matches)
    }
}

fn main() {
    // CLI Arguments parsing
    let matches = App::new("get621")
        .version("1.2.0_pre1")
        .author("nasso <nassomails ~ at ~ gmail {dot} com>")
        .about("E621/926 command line tool")
        .arg(
            Arg::with_name("children")
                .short("c")
                .long("children")
                .conflicts_with("parents")
                .help("Take the children of search results"),
        )
        .arg(
            Arg::with_name("limit")
                .short("l")
                .long("limit")
                .default_value("1")
                .takes_value(true)
                .validator(|v| valid_parse::<u64>(&v, "Must be a positive integer."))
                .help("Maximum search result count"),
        )
        .arg(
            Arg::with_name("parents")
                .short("p")
                .long("parents")
                .conflicts_with("children")
                .help("Take the parent post of each search result, if any"),
        )
        .arg(
            Arg::with_name("pool_id")
                .short("P")
                .long("pool")
                .takes_value(true)
                .validator(|v| valid_parse::<u64>(&v, "Must be a positive integer."))
                .help("Search for posts in the given pool ID (ordered)"),
        )
        .arg(
            Arg::with_name("save")
                .short("s")
                .long("save")
                .help("Download every result to ./<post_id>.<ext>"),
        )
        .arg(
            Arg::with_name("output_mode")
                .short("o")
                .long("output")
                .takes_value(true)
                .default_value("verbose")
                .validator(output_mode_check)
                .help("Set output mode; one of: id, json, raw, verbose, none"),
        )
        .arg(
            Arg::with_name("tags")
                .index(1)
                .multiple(true)
                .allow_hyphen_values(true)
                .conflicts_with("pool_id")
                .help("Search tags"),
        )
        .subcommand(
            SubCommand::with_name("reverse")
                .about("E621/926 reverse searching utils")
                .arg(
                    Arg::with_name("source")
                        .index(1)
                        .allow_hyphen_values(true)
                        .required(true)
                        .help("File or folder to reverse search; can be a glob pattern"),
                )
                .arg(
                    Arg::with_name("similarity")
                        .short("S")
                        .long("similarity")
                        .takes_value(true)
                        .default_value("90")
                        .validator(|v| valid_parse::<f64>(&v, "Must be a floating point value."))
                        .help("Set the similarity threshold for matching posts."),
                )
                .arg(
                    Arg::with_name("save")
                        .short("s")
                        .long("save")
                        .help("Download all matching posts to ./<post_id>.<ext>"),
                )
                .arg(
                    Arg::with_name("output_mode")
                        .short("o")
                        .long("output")
                        .takes_value(true)
                        .default_value("verbose")
                        .validator(output_mode_check)
                        .help("Set output mode; one of: id, json, raw, verbose, none"),
                ),
        )
        .get_matches();

    ::std::process::exit(match run_app(&matches) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    })
}
