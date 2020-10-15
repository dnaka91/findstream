use askama::Template;
use chrono::prelude::*;

use crate::twitch::Stream;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index;

#[derive(Template)]
#[template(path = "results.html")]
pub struct Results {
    query: String,
    responses: Vec<Stream>,
    error: bool,
}

impl Results {
    pub fn new(query: String, responses: Vec<Stream>) -> Self {
        Self {
            query,
            responses,
            error: false,
        }
    }

    pub const fn error() -> Self {
        Self {
            query: String::new(),
            responses: Vec::new(),
            error: true,
        }
    }
}

fn since_now(value: &Option<DateTime<Utc>>) -> String {
    if let Some(value) = value {
        let duration = Utc::now() - *value;
        format!(
            "{} hours {} minutes",
            duration.num_hours(),
            duration.num_minutes() % 60
        )
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
