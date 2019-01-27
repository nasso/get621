extern crate chrono;
extern crate reqwest;
extern crate urlencoding;

use std::io::Write;

use chrono::DateTime;
use chrono::offset::Utc;

static LIST_HARD_LIMIT: u64 = 320;

pub enum Error {
	MaxLimit(u64),
	Http,
	Serial,
	Redirect,
	Server,
	Other(String),
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub enum PostStatus<'a> {
	Active,
	Flagged,
	Pending,
	Deleted(&'a str),
}

pub enum PostRating {
	Safe,
	Questionnable,
	Explicit,
}

pub enum PostFormat {
	JPG,
	PNG,
	GIF,
	SWF,
	WEBM,
}

pub struct Post<'a> {
	pub raw: &'a str,
	
	pub id: u64,
	pub md5: Option<&'a str>,
	pub status: PostStatus<'a>,
	
	pub author: &'a str,
	pub creator_id: u64,
	pub created_at: DateTime<Utc>,
	
	pub artist: Vec<&'a str>,
	pub tags: Vec<&'a str>,
	pub rating: PostRating,
	pub description: &'a str,
	
	pub parent_id: Option<u64>,
	pub children: Option<Vec<u64>>,
	pub sources: Option<Vec<&'a str>>,
	
	pub has_notes: bool,
	pub has_comments: bool,
	
	pub fav_count: u64,
	pub score: i64,
	
	pub file_url: &'a str,
	pub file_ext: Option<PostFormat>,
	pub file_size: Option<u64>,
	
	pub width: u64,
	pub height: u64,
	
	pub sample_url: Option<&'a str>,
	pub sample_width: Option<u64>,
	pub sample_height: Option<u64>,
	
	pub preview_url: &'a str,
	pub preview_width: Option<u64>,
	pub preview_height: Option<u64>,
}

fn get_json(url: &str) -> Result<String> {
	match reqwest::get(url) {
		Ok(mut res) => {
			if res.status().is_success() {
				Ok(res.text().unwrap())
			} else {
				Err(
					if res.status().is_server_error() {
						Error::Server
					} else {
						Error::Other(format!("{:?}", res.status()))
					}
				)
			}
		},
		Err(e) => Err({
			if e.is_http() {
				Error::Http
			} else if e.is_serialization() {
				Error::Serial
			} else if e.is_redirect() {
				Error::Redirect
			} else if e.is_server_error() {
				Error::Server
			} else {
				Error::Other(format!("{:?}", e))
			}
		}),
	}
}

pub fn list<'n>(q: &[&str], limit: u64) -> Result<Vec<Post<'n>>> {
	let query_str = q.join(" ");
	let query_str_url = urlencoding::encode(&query_str);
	let ordered = q.iter().any(|t| t.starts_with("order:"));
	
	if ordered {
		if limit > LIST_HARD_LIMIT {
			return Err(Error::MaxLimit(LIST_HARD_LIMIT));
		}
		
		let body = get_json(&format!(
			"https://e621.net/post/index.json?limit={}&tags={}",
			limit,
			query_str_url
		))?;
		
		println!("{}", body);
	} else {
		// let body = reqwest::get(format!("https://e621.net/post/index.json?limit={}, "));
	}
	
	
	let posts = Vec::new();
	
	Ok(posts)
}
