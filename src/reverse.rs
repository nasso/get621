use crate::common::{
    self, expand_paths, output_mode_check, output_posts, save_posts, valid_parse, OutputMode,
};
use clap::{Arg, ArgMatches};
use get621::Get621;
use regex::Regex;
use reqwest::{self, multipart};
use scraper::{Html, Selector};

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
            .help("Set the similarity threshold for matching posts"),
        Arg::with_name("save")
            .short("s")
            .long("save")
            .help("Download all matching posts to ./<post_id>.<ext>"),
        Arg::with_name("output_mode")
            .short("o")
            .long("output")
            .takes_value(true)
            .default_value("verbose")
            .validator(output_mode_check)
            .help("Set output mode; one of: id, json, raw, verbose"),
    ]
}

// get621 reverse ...
pub fn run(matches: &ArgMatches) -> common::Result<()> {
    lazy_static! {
        static ref CLIENT: reqwest::Client =
            reqwest::Client::builder().timeout(None).build().unwrap();
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

    let arg_source = matches.values_of("source").unwrap().collect::<Vec<_>>();
    let arg_similarity = matches.value_of("similarity").unwrap().parse().unwrap();
    let arg_outputmode = matches.value_of("output_mode").unwrap();

    let is_verbose = match arg_outputmode.into() {
        OutputMode::Verbose => true,
        _ => false,
    };

    for path in expand_paths(&arg_source)?
        .into_iter()
        .filter(|path| path.is_file())
    {
        if is_verbose {
            println!("Looking for {}", path.display());
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
            save_posts(&g6, &posts, None)?;
        }

        if is_verbose {
            println!();
        }
    }

    Ok(())
}
