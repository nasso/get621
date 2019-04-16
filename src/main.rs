use clap::{App, Arg, ArgMatches, SubCommand};
use get621::Get621;
use globwalk;
use reqwest::{self, multipart};
use scraper::{Html, Selector};
use std::{fmt, fs::File, io, str::FromStr};

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

fn valid_parse<T: FromStr>(v: &str, emsg: &str) -> Result<(), String> {
    match v.parse::<T>() {
        Ok(_) => Ok(()),
        Err(_) => Err(emsg.to_string()),
    }
}

fn output_mode_check(v: String) -> Result<(), String> {
    if v == "id" || v == "json" || v == "raw" || v == "verbose" || v == "none" {
        Ok(())
    } else {
        Err("Must be one of: id, json, raw, verbose, none".to_string())
    }
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
    match matches.value_of("output_mode") {
        Some("id") => {
            posts.iter().for_each(|p| println!("{}", p.id));
        }

        Some("json") => {
            println!(
                "[{}]",
                posts
                    .iter()
                    .map(|p| p.raw.clone())
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }

        Some("raw") => {
            let mut stdout = io::stdout();

            for p in posts.iter().filter(|p| !p.status.is_deleted()) {
                g6.download(p, &mut stdout)?;
            }
        }

        _ => {
            println!(
                "{}",
                posts
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join("\n----------------\n")
            );
        }
    }

    if matches.is_present("save") {
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
    }

    Ok(())
}

// get621 reverse ...
fn run_reverse(matches: &ArgMatches) -> Result<(), Error> {
    let client = reqwest::Client::new();

    let result_divs_selector =
        Selector::parse("#pages > div:not(:first-child):not(#show1):not(#more1)").unwrap();

    for entry in globwalk::glob(matches.value_of("source").unwrap())?
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
    {
        let path = entry.path();

        println!("Looking for {}", path.to_string_lossy());

        let form = multipart::Form::new()
            .text("service[]", "0")
            .text("MAX_FILE_SIZE", "8388608")
            .file("file", path)?
            .text("url", "");

        let mut resp = client
            .post("http://iqdb.harry.lu/")
            .multipart(form)
            .send()?;

        let doc = Html::parse_document(&resp.text()?);

        for result in doc.select(&result_divs_selector) {
            println!("found: {}", result.value().name.local);
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
