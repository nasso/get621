use get621::Post;
use glob;
use lazy_static::lazy_static;
use reqwest;
use std::{fmt, fs::File, io, path::PathBuf, str::FromStr};

lazy_static! {
    pub static ref CLIENT: reqwest::Client =
        reqwest::Client::builder().timeout(None).build().unwrap();
}

pub enum Error {
    Get621Error(get621::Error),
    IOError(io::Error),
    GlobError(glob::GlobError),
    PatternError(glob::PatternError),
    ReqwestError(reqwest::Error),
    Http(u16),
    Redirect(String),
    CannotSendRequest(String),
    Download(String),
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

impl From<glob::GlobError> for Error {
    fn from(e: glob::GlobError) -> Self {
        Error::GlobError(e)
    }
}

impl From<glob::PatternError> for Error {
    fn from(e: glob::PatternError) -> Self {
        Error::PatternError(e)
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
            Error::PatternError(e) => write!(f, "{}", e),
            Error::ReqwestError(e) => write!(f, "{}", e),
            Error::Http(code) => write!(f, "HTTP error: {}", code),
            Error::Redirect(msg) => write!(f, "Redirect error: {}", msg),
            Error::CannotSendRequest(msg) => write!(f, "Couldn't send request: {}", msg),
            Error::Download(msg) => write!(f, "Error when downloading the post: {}", msg),
        }
    }
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

/// Downloads the given URL to `writer`.
///
/// On success, the total number of bytes that were copied from `reader` to `writer` is returned.
pub fn download<W, U>(url: U, writer: &mut W) -> Result<u64>
where
    U: reqwest::IntoUrl,
    W: ?Sized + io::Write,
{
    match CLIENT.get(url).send() {
        Ok(mut res) => {
            if res.status().is_success() {
                match res.copy_to(writer) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(Error::Download(format!("{:?}", e))),
                }
            } else {
                Err(Error::Http(res.status().as_u16()))
            }
        }

        Err(e) => {
            if e.is_redirect() {
                Err(Error::Redirect(format!("{:?}", e)))
            } else {
                Err(Error::CannotSendRequest(format!("{:?}", e)))
            }
        }
    }
}

// output the posts
pub fn output_posts<T: Into<OutputMode>>(posts: &Vec<Post>, mode: T) -> Result<()> {
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
                download(&p.file_url, &mut stdout)?;
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
                        .map(|p| p.to_string())
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

        download(&p.file_url, &mut file)?;
    }

    Ok(())
}
