use askama::Template;
use chrono::prelude::*;

use self::lang::translate_iso_639_1;
use crate::twitch::Stream;

mod lang;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index;

#[derive(Template)]
#[template(path = "results.html")]
pub struct Results {
    query_words: Vec<String>,
    streams: Vec<Stream>,
    error: bool,
}

impl Results {
    pub fn new(query_words: Vec<String>, streams: Vec<Stream>) -> Self {
        Self {
            query_words,
            streams,
            error: false,
        }
    }

    pub const fn error() -> Self {
        Self {
            query_words: Vec::new(),
            streams: Vec::new(),
            error: true,
        }
    }
}

fn since_now(value: &Option<DateTime<Utc>>) -> String {
    if let Some(value) = value {
        let duration = Utc::now() - *value;
        let mut buf = String::new();

        match duration.num_days() {
            0 => {}
            1 => buf.push_str("1 day"),
            d => buf.push_str(&format!("{} days", d)),
        }

        match duration.num_hours() % 24 {
            0 => {}
            1 => buf.push_str(" 1 hour"),
            h => buf.push_str(&format!(" {} hours", h)),
        }

        match duration.num_minutes() % 60 {
            0 => {}
            1 => buf.push_str(" 1 minute"),
            m => buf.push_str(&format!(" {} minutes", m)),
        }

        buf
    } else {
        String::new()
    }
}

#[allow(clippy::pedantic)]
fn sized(value: &str, width: &u32, height: &u32) -> String {
    value
        .replace("{width}", &width.to_string())
        .replace("{height}", &height.to_string())
}
