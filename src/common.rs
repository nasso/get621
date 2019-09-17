use clap::ArgMatches;
use custom_error::custom_error;
use glob;
use lazy_static::lazy_static;
use reqwest;
use rs621::{
    client::Client as Rs621Client,
    error::Result as Rs621Result,
    post::{Post, PostStatus},
};
use std::{fmt, fs::File, io, path::PathBuf, str::FromStr};

lazy_static! {
    pub static ref CLIENT: reqwest::Client =
        reqwest::Client::builder().timeout(None).build().unwrap();
}

custom_error! { pub Error
    Rs621Error{source:rs621::error::Error} = "{source}",
    IOError{source:io::Error} = "{source}",
    GlobError{source:glob::GlobError} = "{source}",
    PatternError{source:glob::PatternError} = "{source}",
    ReqwestError{source:reqwest::Error} = "{source}",
    Http{code:u16} = "HTTP error: {code}",
    Redirect{desc:String} = "Redirect error: {desc}",
    CannotSendRequest{desc:String} = "Couldn't send request: {desc}",
    Download{desc:String} = "Error when downloading the post: {desc}",
}

pub type Result<T> = std::result::Result<T, Error>;

pub enum OutputMode {
    Id,
    Json,
    Raw,
    Verbose,
}

impl From<&str> for OutputMode {
    fn from(s: &str) -> Self {
        match s {
            "id" => OutputMode::Id,
            "json" => OutputMode::Json,
            "raw" => OutputMode::Raw,
            "verbose" => OutputMode::Verbose,
            _ => panic!("Invalid output mode: {}", s),
        }
    }
}

// asserts that a string can be parsed into a type
pub fn valid_parse<T: FromStr>(v: &str, emsg: &str) -> std::result::Result<(), String> {
    match v.parse::<T>() {
        Ok(_) => Ok(()),
        Err(_) => Err(emsg.to_string()),
    }
}

pub fn output_mode_check(v: String) -> std::result::Result<(), String> {
    if v == "id" || v == "json" || v == "raw" || v == "verbose" {
        Ok(())
    } else {
        Err(String::from("Invalid output mode."))
    }
}

// process a list of paths into another list, expanding glob patterns and folders
pub fn expand_paths<S: AsRef<str>>(patterns: &[S]) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();

    for p in patterns.into_iter() {
        let p = p.as_ref();

        for entry in glob::glob(p)?.filter_map(std::result::Result::ok) {
            results.push(entry.canonicalize()?);
        }
    }

    Ok(results)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PostMapMode {
    Parents,
    Children,
    None,
}

impl From<&ArgMatches<'_>> for PostMapMode {
    fn from(matches: &ArgMatches) -> Self {
        if matches.is_present("parents") {
            PostMapMode::Parents
        } else if matches.is_present("children") {
            PostMapMode::Children
        } else {
            PostMapMode::None
        }
    }
}

pub fn post_map(
    client: &Rs621Client,
    mode: PostMapMode,
    post_iter: impl Iterator<Item = Rs621Result<Post>>,
) -> Result<Vec<Post>> {
    Ok(match mode {
        PostMapMode::None => post_iter
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?,
        PostMapMode::Parents => post_iter
            .filter_map(|p| match p {
                Ok(p) => p.parent_id.map(|id| client.get_post(id)),
                Err(e) => Some(Err(e)),
            })
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?,
        PostMapMode::Children => {
            // Vec holding all the children of all the posts
            let mut all_children = Vec::new();

            // collect the children of every post in a Vec
            for post in post_iter {
                let post_children = post?
                    .children
                    .iter()
                    .map(|id| client.get_post(*id).map_err(|e| e.into()))
                    .collect::<Result<Vec<_>>>()?;

                // add it to the big Vec
                all_children.extend(post_children);
            }

            all_children
        }
    })
}

/// Downloads the given URL to `writer`.
///
/// On success, the total number of bytes that were copied from `reader` to `writer` is returned.
pub fn download<W, U>(url: U, writer: &mut W) -> Result<u64>
where
    U: reqwest::IntoUrl,
    W: ?Sized + io::Write,
{
    let mut res = CLIENT.get(url).send()?;

    if res.status().is_success() {
        Ok(res.copy_to(writer)?)
    } else {
        Err(Error::Http {
            code: res.status().as_u16(),
        })
    }
}

#[derive(Debug)]
struct DisplayablePost<'a>(&'a Post);

impl fmt::Display for DisplayablePost<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let PostStatus::Deleted(ref reason) = &self.0.status {
            writeln!(f, "#{} (deleted: {})", self.0.id, reason)?;
        } else {
            write!(f, "#{} by ", self.0.id)?;

            let artist_count = self.0.artists.len();
            for i in 0..artist_count {
                match artist_count - i {
                    1 => writeln!(f, "{}", self.0.artists[i])?,
                    2 => write!(f, "{} and ", self.0.artists[i])?,
                    _ => write!(f, "{}, ", self.0.artists[i])?,
                }
            }
        }

        writeln!(f, "Rating: {}", self.0.rating)?;

        writeln!(f, "Score: {}", self.0.score)?;
        writeln!(f, "Favs: {}", self.0.fav_count)?;

        if let Some(ref t) = self.0.file_ext {
            writeln!(f, "Type: {}", t)?;
        }

        writeln!(f, "Created at: {}", self.0.created_at)?;
        writeln!(f, "Tags: {}", self.0.tags.join(", "))?;
        write!(f, "Description: {}", self.0.description)?;

        Ok(())
    }
}

// output the posts
pub fn output_posts(posts: &Vec<Post>, mode: OutputMode) -> Result<()> {
    match mode {
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

            for p in posts.iter().filter(|p| !p.is_deleted()) {
                download(p.file_url.as_ref().unwrap(), &mut stdout)?;
            }

            Ok(())
        }

        OutputMode::Verbose => {
            if posts.is_empty() {
                println!("No post found.");
            } else {
                println!(
                    "{}",
                    posts
                        .iter()
                        .map(|p| DisplayablePost(p).to_string())
                        .collect::<Vec<_>>()
                        .join("\n----------------\n")
                );
            }

            Ok(())
        }
    }
}

// save the posts
pub fn save_posts(posts: &Vec<Post>, pool_id: Option<u64>) -> Result<()> {
    for (i, p) in posts.iter().filter(|p| !p.is_deleted()).enumerate() {
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

        download(p.file_url.as_ref().unwrap(), &mut file)?;
    }

    Ok(())
}
