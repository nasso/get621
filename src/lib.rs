extern crate chrono;
extern crate reqwest;
extern crate urlencoding;

use std::{cmp, fmt};

use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};

use chrono::{offset::Utc, DateTime, TimeZone};

use serde_json;

static LIST_HARD_LIMIT: usize = 320;
const REQ_COOLDOWN_DURATION: ::std::time::Duration = ::std::time::Duration::from_secs(1);

pub type JsonValue = serde_json::Value;
pub type Result<T> = ::std::result::Result<T, Error>;

pub enum Error {
    AboveLimit(usize, usize),
    Http(u16),
    Serial(String),
    Redirect(String),
    CannotSendRequest(String),
    CannotCreateClient(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::AboveLimit(limit, max) => write!(
                f,
                "{} is above the max limit for ordered queries ({})",
                limit, max
            ),
            Error::Http(code) => write!(f, "HTTP error: {}", code),
            Error::Serial(msg) => write!(f, "Serialization error: {}", msg),
            Error::Redirect(msg) => write!(f, "Redirect error: {}", msg),
            Error::CannotSendRequest(msg) => write!(f, "Couldn't send request: {}", msg),
            Error::CannotCreateClient(msg) => write!(f, "Couldn't create client: {}", msg),
        }
    }
}

pub enum PostStatus {
    Active,
    Flagged,
    Pending,
    Deleted(String),
}

impl PostStatus {
    pub fn is_deleted(&self) -> bool {
        match self {
            PostStatus::Deleted(_) => true,
            _ => false,
        }
    }
}

pub enum PostRating {
    Safe,
    Questionnable,
    Explicit,
}

impl fmt::Display for PostRating {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PostRating::Explicit => write!(f, "Explicit"),
            PostRating::Questionnable => write!(f, "Questionnable"),
            PostRating::Safe => write!(f, "Safe"),
        }
    }
}

pub enum PostFormat {
    JPG,
    PNG,
    GIF,
    SWF,
    WEBM,
}

impl fmt::Display for PostFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PostFormat::JPG => write!(f, "jpg"),
            PostFormat::PNG => write!(f, "png"),
            PostFormat::GIF => write!(f, "gif"),
            PostFormat::SWF => write!(f, "swf"),
            PostFormat::WEBM => write!(f, "webm"),
        }
    }
}

pub struct Post {
    pub raw: String,

    pub id: u64,
    pub md5: Option<String>,
    pub status: PostStatus,

    pub author: String,
    pub creator_id: Option<u64>,
    pub created_at: DateTime<Utc>,

    pub artist: Vec<String>,
    pub tags: Vec<String>,
    pub rating: PostRating,
    pub description: String,

    pub parent_id: Option<u64>,
    pub children: Option<Vec<u64>>,
    pub sources: Option<Vec<String>>,

    pub has_notes: bool,
    pub has_comments: bool,

    pub fav_count: u64,
    pub score: i64,

    pub file_url: String,
    pub file_ext: Option<PostFormat>,
    pub file_size: Option<u64>,

    pub width: u64,
    pub height: u64,

    pub sample_url: Option<String>,
    pub sample_width: Option<u64>,
    pub sample_height: Option<u64>,

    pub preview_url: String,
    pub preview_width: Option<u64>,
    pub preview_height: Option<u64>,
}

impl fmt::Display for Post {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let PostStatus::Deleted(ref reason) = &self.status {
            writeln!(f, "#{} (deleted: {})", self.id, reason)?;
        } else {
            write!(f, "#{} by ", self.id)?;

            let artist_count = self.artist.len();
            for i in 0..artist_count {
                match artist_count - i {
                    1 => writeln!(f, "{}", self.artist[i])?,
                    2 => write!(f, "{} and ", self.artist[i])?,
                    _ => write!(f, "{}, ", self.artist[i])?,
                }
            }
        }

        writeln!(f, "Rating: {}", self.rating)?;

        writeln!(f, "Score: {}", self.score)?;
        writeln!(f, "Favs: {}", self.fav_count)?;

        if let Some(ref t) = self.file_ext {
            writeln!(f, "Type: {}", t)?;
        }

        writeln!(f, "Created at: {}", self.created_at)?;
        writeln!(f, "Tags: {}", self.tags.join(", "))?;
        write!(f, "Description: {}", self.description)?;

        Ok(())
    }
}

impl From<&JsonValue> for Post {
    fn from(v: &JsonValue) -> Self {
        Post {
            raw: v.to_string(),

            id: v["id"].as_u64().unwrap(),
            md5: v["md5"].as_str().map(String::from),
            status: match v["status"].as_str() {
                Some("active") => PostStatus::Active,
                Some("flagged") => PostStatus::Flagged,
                Some("pending") => PostStatus::Pending,
                Some("deleted") => {
                    PostStatus::Deleted(v["delreason"].as_str().unwrap().to_string())
                }
                _ => unreachable!(),
            },

            author: v["author"].as_str().unwrap().to_string(),
            creator_id: v["creator_id"].as_u64(),
            created_at: Utc.timestamp(
                v["created_at"]["s"].as_i64().unwrap(),
                v["created_at"]["n"].as_u64().unwrap() as u32,
            ),

            artist: v["artist"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
            tags: v["tags"]
                .as_str()
                .unwrap()
                .split_whitespace()
                .map(String::from)
                .collect(),
            rating: match v["rating"].as_str().unwrap() {
                "e" => PostRating::Explicit,
                "q" => PostRating::Questionnable,
                "s" => PostRating::Safe,
                _ => unreachable!(),
            },
            description: v["description"].as_str().unwrap().to_string(),

            parent_id: v["parent_id"].as_u64(),
            children: v["children"].as_str().map(|c| {
                if c.is_empty() {
                    Vec::new()
                } else {
                    c.split(',').map(|id| id.parse().unwrap()).collect()
                }
            }),

            sources: v["children"]
                .as_array()
                .map(|v| v.iter().map(|v| v.as_str().unwrap().to_string()).collect()),

            has_notes: v["has_notes"].as_bool().unwrap(),
            has_comments: v["has_comments"].as_bool().unwrap(),

            fav_count: v["fav_count"].as_u64().unwrap(),
            score: v["score"].as_i64().unwrap(),

            file_url: v["file_url"].as_str().unwrap().to_string(),
            file_ext: v["file_ext"].as_str().map(|v| match v {
                "jpg" => PostFormat::JPG,
                "png" => PostFormat::PNG,
                "gif" => PostFormat::GIF,
                "swf" => PostFormat::SWF,
                "webm" => PostFormat::WEBM,
                _ => unreachable!(),
            }),
            file_size: v["file_size"].as_u64(),

            width: v["width"].as_u64().unwrap(),
            height: v["height"].as_u64().unwrap(),

            sample_url: v["sample_url"].as_str().map(String::from),
            sample_width: v["sample_width"].as_u64(),
            sample_height: v["sample_height"].as_u64(),

            preview_url: v["preview_url"].as_str().unwrap().to_string(),
            preview_width: v["preview_width"].as_u64(),
            preview_height: v["preview_height"].as_u64(),
        }
    }
}

pub struct Get621 {
    client: Client,
}

impl Get621 {
    /// Create a get621 client
    pub fn init() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_static("get621 (by yann-the-leopard on e621)"),
        );

        match Client::builder()
            .timeout(None)
            .default_headers(headers)
            .build()
        {
            Ok(c) => Ok(Get621 { client: c }),
            Err(e) => Err(Error::CannotCreateClient(format!("{:?}", e))),
        }
    }

    fn get_json<U: reqwest::IntoUrl>(&self, url: U) -> Result<JsonValue> {
        // Wait first to make sure we're not exceeding the limit
        ::std::thread::sleep(REQ_COOLDOWN_DURATION);

        match self.client.get(url).send() {
            Ok(mut res) => {
                if res.status().is_success() {
                    match res.json() {
                        Ok(v) => Ok(v),
                        Err(e) => Err(Error::Serial(format!("{:?}", e))),
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

    pub fn get_post(&self, id: u64) -> Result<Post> {
        let body = self.get_json(&format!("https://e621.net/post/show.json?id={}", id))?;

        Ok(Post::from(&body))
    }

    pub fn pool(&self, pool_id: u64) -> Result<Vec<Post>> {
        let mut body = self.get_json(&format!("https://e621.net/pool/show.json?id={}", pool_id))?;

        let mut page = 1;
        let mut post_array = body["posts"].as_array().unwrap();

        let mut posts = Vec::new();

        loop {
            for p in post_array.iter() {
                posts.push(Post::from(p));
            }

            page += 1;
            body = self.get_json(&format!(
                "https://e621.net/pool/show.json?id={}&page={}",
                pool_id, page
            ))?;
            post_array = body["posts"].as_array().unwrap();

            if post_array.is_empty() {
                break;
            }
        }

        Ok(posts)
    }

    pub fn list(&self, q: &[&str], limit: usize) -> Result<Vec<Post>> {
        let query_str = q.join(" ");
        let query_str_url = urlencoding::encode(&query_str);
        let ordered = q.iter().any(|t| t.starts_with("order:"));

        let mut posts = Vec::new();

        if ordered {
            if limit > LIST_HARD_LIMIT {
                return Err(Error::AboveLimit(limit, LIST_HARD_LIMIT));
            }

            let body = self.get_json(&format!(
                "https://e621.net/post/index.json?limit={}&tags={}",
                limit, query_str_url
            ))?;

            for p in body.as_array().unwrap().iter() {
                posts.push(Post::from(p));
            }
        } else {
            let mut lowest_id = None;

            while posts.len() < limit {
                let left = limit - posts.len();
                let batch = cmp::min(left, LIST_HARD_LIMIT);

                let body = self.get_json(&format!(
                    "https://e621.net/post/index.json?limit={}&tags={}{}",
                    batch,
                    query_str_url,
                    if let Some(i) = lowest_id {
                        format!("&before_id={}", i)
                    } else {
                        "".to_string()
                    }
                ))?;

                let post_array = body.as_array().unwrap();

                if post_array.is_empty() {
                    break;
                }

                for p in post_array.iter() {
                    let post = Post::from(p);

                    if let Some(i) = lowest_id {
                        lowest_id = Some(cmp::min(i, post.id));
                    } else {
                        lowest_id = Some(post.id);
                    }

                    posts.push(post);
                }
            }
        }

        Ok(posts)
    }
}
