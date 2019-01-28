extern crate chrono;
extern crate reqwest;
extern crate urlencoding;

use std::fmt;

use reqwest::{
	Client,
	IntoUrl,
	header::{
		self,
		HeaderMap,
		HeaderValue
	},
};

use chrono::{
	TimeZone,
	DateTime,
	offset::Utc,
};

use serde_json;

static LIST_HARD_LIMIT: u64 = 320;
const REQ_COOLDOWN_DURATION: ::std::time::Duration = ::std::time::Duration::from_secs(1);

pub type JsonValue = serde_json::Value;
pub type Result<T> = ::std::result::Result<T, Error>;

pub enum Error {
	MaxLimit(u64),
	Http(u16),
	Serial(String),
	Redirect(String),
	CannotSendRequest(String),
	CannotCreateClient(String),
}

pub enum PostStatus {
	Active,
	Flagged,
	Pending,
	Deleted(String),
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

pub struct Post {
	pub raw: String,
	
	pub id: u64,
	pub md5: Option<String>,
	pub status: PostStatus,
	
	pub author: String,
	pub creator_id: u64,
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
		write!(f, "#{} by ", self.id)?;
		
		let artist_count = self.artist.len();
		for i in 0..artist_count {
			match artist_count - i {
				1 => writeln!(f, "{}", self.artist[i])?,
				2 => write!(f, "{} and ", self.artist[i])?,
				_ => write!(f, "{}, ", self.artist[i])?,
			}
		}
		
		writeln!(f, "Rating: {}", match self.rating {
			PostRating::Explicit => "Explicit",
			PostRating::Questionnable => "Questionnable",
			PostRating::Safe => "Safe",
		})?;
		
		writeln!(f, "Score: {}", self.score)?;
		writeln!(f, "Favs: {}", self.fav_count)?;
		if let Some(ref t) = self.file_ext {
			writeln!(f, "Type: {}", match t {
				PostFormat::JPG => "jpg",
				PostFormat::PNG => "png",
				PostFormat::GIF => "gif",
				PostFormat::SWF => "swf",
				PostFormat::WEBM => "webm",
			})?;
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
				Some("deleted") => PostStatus::Deleted(v["delreason"].as_str().unwrap().to_string()),
				_ => unreachable!(),
			},
			
			author: v["author"].as_str().unwrap().to_string(),
			creator_id: v["creator_id"].as_u64().unwrap(),
			created_at: Utc.timestamp(
				v["created_at"]["s"].as_i64().unwrap(),
				v["created_at"]["n"].as_u64().unwrap() as u32
			),
			
			artist: v["artist"]
			        .as_array().unwrap()
			        .iter().map(|v| v.as_str().unwrap().to_string())
			        .collect(),
			tags: v["tags"]
			      .as_str().unwrap()
			      .split_whitespace().map(String::from)
			      .collect(),
			rating: match v["rating"].as_str().unwrap() {
				"e" => PostRating::Explicit,
				"q" => PostRating::Questionnable,
				"s" => PostRating::Safe,
				_ => unreachable!(),
			},
			description: v["description"].as_str().unwrap().to_string(),
			
			parent_id: v["parent_id"].as_u64(),
			children: v["children"]
			          .as_array()
			          .map(|v| v
			                   .iter()
			                   .map(|v| v.as_u64().unwrap())
			                   .collect()),
			sources: v["children"]
			         .as_array()
			         .map(|v| v
			                  .iter()
			                  .map(|v| v.as_str().unwrap().to_string())
			                  .collect()),
			
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
			HeaderValue::from_static("get621 (by yann-the-leopard on e621)")
		);
		
		match Client::builder().timeout(None).default_headers(headers).build() {
			Ok(c) => {
				Ok(Get621 {
					client: c,
				})
			},
			
			Err(e) => {
				Err(Error::CannotCreateClient(format!("{:?}", e)))
			},
		}
	}
	
	fn get_json<U: IntoUrl>(&self, url: U) -> Result<JsonValue> {
		// Wait first to make sure we're not exceeding the limit
		::std::thread::sleep(REQ_COOLDOWN_DURATION);
		
		match self.client.get(url).send() {
			Ok(mut res) => {
				if res.status().is_success() {
					match res.json() {
						Ok(v) => Ok(v),
						Err(e) => {
							Err(Error::Serial(format!("{:?}", e)))
						},
					}
				} else {
					Err(Error::Http(res.status().as_u16()))
				}
			},
			
			Err(e) => {
				if e.is_redirect() {
					Err(Error::Redirect(format!("{:?}", e)))
				} else {
					Err(Error::CannotSendRequest(format!("{:?}", e)))
				}
			},
		}
	}
	
	pub fn list(&self, q: &[&str], limit: u64) -> Result<Vec<Post>> {
		let query_str = q.join(" ");
		let query_str_url = urlencoding::encode(&query_str);
		let ordered = q.iter().any(|t| t.starts_with("order:"));
		
		let mut posts = Vec::new();
		
		if ordered {
			if limit > LIST_HARD_LIMIT {
				return Err(Error::MaxLimit(LIST_HARD_LIMIT));
			}
			
			let body = self.get_json(&format!(
				"https://e621.net/post/index.json?limit={}&tags={}",
				limit,
				query_str_url
			))?;
			
			for p in body.as_array().unwrap().iter() {
				posts.push(Post::from(p));
			}
		} else {
			// let body = reqwest::get(format!("https://e621.net/post/index.json?limit={}, "));
		}
		
		Ok(posts)
	}
}
