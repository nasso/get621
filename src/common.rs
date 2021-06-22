use clap::ArgMatches;
use futures::{pin_mut, stream::StreamExt, Stream};
use glob;
use lazy_static::lazy_static;
use reqwest;
use rs621::{
    client::Client as Rs621Client,
    error::Result as Rs621Result,
    post::{Post, PostFileExtension, PostRating},
};
use std::{fmt, fs::File, io, path::PathBuf, str::FromStr};

lazy_static! {
    pub static ref CLIENT: reqwest::Client = reqwest::Client::builder().build().unwrap();
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API error: {0}")]
    Rs621Error(#[from] rs621::error::Error),
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
    #[error("Glob pattern error: {0}")]
    GlobError(#[from] glob::GlobError),
    #[error("Pattern error: {0}")]
    PatternError(#[from] glob::PatternError),
    #[error("Network error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("JSON parse error: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(u16),
    #[error("Pool not found")]
    PoolNotFound,
    #[error("Couldn't get the authenticity token")]
    AuthTokenNotFound,
    #[error("The IQDB query failed or returned unknown results")]
    IqdbQueryError,
    #[error("A post is missing a file URL")]
    MissingFileUrl,
}

pub type Result<T> = std::result::Result<T, Error>;

pub enum OutputMode {
    Id,
    Raw,
    Verbose,
}

impl From<&str> for OutputMode {
    fn from(s: &str) -> Self {
        match s {
            "id" => OutputMode::Id,
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
    if v == "id" || v == "raw" || v == "verbose" {
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

pub async fn post_map(
    client: &Rs621Client,
    mode: PostMapMode,
    mut post_stream: impl Stream<Item = Rs621Result<Post>> + Unpin,
) -> Result<Vec<Post>> {
    Ok(match mode {
        PostMapMode::None => post_stream
            .map(|r| r.map_err(|e| Error::from(e)))
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?,
        PostMapMode::Parents => post_stream
            .filter_map(|p| async move {
                match p {
                    Ok(p) => {
                        if let Some(parent) = p.relationships.parent_id {
                            client.get_posts(&[parent]).next().await
                        } else {
                            None
                        }
                    }
                    Err(e) => Some(Err(e)),
                }
            })
            .map(|r| r.map_err(|e| Error::from(e)))
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?,
        PostMapMode::Children => {
            // Vec holding all the children of all the posts
            let mut all_children = Vec::new();

            // collect the children of every post in a Vec
            while let Some(post) = post_stream.next().await {
                let post = post?;

                all_children.reserve(post.relationships.children.len());

                for child in post.relationships.children.iter() {
                    if let Some(child_post) = client.get_posts(&[*child]).next().await {
                        all_children.push(child_post?);
                    }
                }
            }

            all_children
        }
    })
}

/// Downloads the given URL to `writer`.
///
/// On success, the total number of bytes that were copied from `reader` to `writer` is returned.
pub async fn download<W, U>(url: U, writer: &mut W) -> Result<u64>
where
    U: reqwest::IntoUrl,
    W: ?Sized + io::Write,
{
    let mut res = CLIENT.get(url).send().await?;

    if res.status().is_success() {
        let mut bytes = 0;

        while let Some(chunk) = res.chunk().await? {
            writer.write_all(&chunk)?;
            bytes += chunk.len() as u64;
        }

        Ok(bytes)
    } else {
        Err(Error::Http(res.status().as_u16()))
    }
}

#[derive(Debug)]
struct DisplayablePost<'a>(&'a Post);

impl fmt::Display for DisplayablePost<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.flags.deleted {
            writeln!(f, "#{} (deleted)", self.0.id)?;
        } else {
            write!(f, "#{} by ", self.0.id)?;

            let artist_count = self.0.tags.artist.len();
            for i in 0..artist_count {
                match artist_count - i {
                    1 => writeln!(f, "{}", self.0.tags.artist[i])?,
                    2 => write!(f, "{} and ", self.0.tags.artist[i])?,
                    _ => write!(f, "{}, ", self.0.tags.artist[i])?,
                }
            }
        }

        writeln!(
            f,
            "Rating: {}",
            match self.0.rating {
                PostRating::Safe => "safe",
                PostRating::Questionable => "questionable",
                PostRating::Explicit => "explicit",
            }
        )?;

        writeln!(
            f,
            "Score: {} (+{}; -{})",
            self.0.score.total,
            self.0.score.up,
            self.0.score.down.abs()
        )?;
        writeln!(f, "Favs: {}", self.0.fav_count)?;

        writeln!(
            f,
            "Type: {}",
            match self.0.file.ext {
                PostFileExtension::Jpeg => "JPEG",
                PostFileExtension::Png => "PNG",
                PostFileExtension::Gif => "GIF",
                PostFileExtension::Swf => "SWF",
                PostFileExtension::WebM => "WEBM",
            }
        )?;

        writeln!(f, "Created at: {}", self.0.created_at)?;
        writeln!(f, "Tags:")?;

        // TODO smartly wrap tags according to the terminal width?

        if !self.0.tags.artist.is_empty() {
            writeln!(f, "  [artist]\n    {}", self.0.tags.artist.join(" "))?;
        }

        if !self.0.tags.lore.is_empty() {
            writeln!(f, "  [lore]\n    {}", self.0.tags.lore.join(" "))?;
        }

        if !self.0.tags.character.is_empty() {
            writeln!(f, "  [character]\n    {}", self.0.tags.character.join(" "))?;
        }

        if !self.0.tags.species.is_empty() {
            writeln!(f, "  [species]\n    {}", self.0.tags.species.join(" "))?;
        }

        if !self.0.tags.general.is_empty() {
            writeln!(f, "  [general]\n    {}", self.0.tags.general.join(" "))?;
        }

        if !self.0.tags.meta.is_empty() {
            writeln!(f, "  [meta]\n    {}", self.0.tags.meta.join(" "))?;
        }

        if !self.0.tags.invalid.is_empty() {
            writeln!(f, "  [invalid]\n    {}", self.0.tags.invalid.join(" "))?;
        }

        write!(f, "Description: {}", self.0.description)?;

        Ok(())
    }
}

// output the posts
pub async fn output_posts(
    mut posts: impl Stream<Item = Post> + Unpin,
    mode: OutputMode,
) -> Result<()> {
    match mode {
        OutputMode::Id => {
            while let Some(post) = posts.next().await {
                println!("{}", post.id);
            }

            Ok(())
        }

        OutputMode::Raw => {
            let results = posts
                .filter_map(|p| async move { p.file.url })
                .then(|url| async move { download(url, &mut io::stdout()).await });

            pin_mut!(results);

            while let Some(res) = results.next().await {
                res?;
            }

            Ok(())
        }

        OutputMode::Verbose => {
            let mut is_empty = true;

            while let Some(post) = posts.next().await {
                if !is_empty {
                    println!("----------------");
                }

                is_empty = false;
                println!("{}", DisplayablePost(&post));
            }

            if is_empty {
                println!("No post found.");
            }

            Ok(())
        }
    }
}

// save the posts
pub async fn save_post(post: &Post, prefix: impl Into<Option<&str>>) -> Result<()> {
    let id = post.id;
    let url = post.file.url.as_ref().ok_or(Error::MissingFileUrl)?;

    let mut file = File::create(format!(
        "{}{}.{}",
        prefix.into().unwrap_or(""),
        id,
        match post.file.ext {
            PostFileExtension::Jpeg => "jpg",
            PostFileExtension::Png => "png",
            PostFileExtension::Gif => "gif",
            PostFileExtension::Swf => "swf",
            PostFileExtension::WebM => "webm",
        }
    ))?;

    download(url, &mut file).await?;

    Ok(())
}
